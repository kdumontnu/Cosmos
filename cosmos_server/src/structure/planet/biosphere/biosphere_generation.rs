use bevy::prelude::*;
use bevy_app_compute::prelude::*;
use bytemuck::{Pod, Zeroable};
use cosmos_core::{
    physics::location::Location,
    structure::{
        block_storage::BlockStorer,
        chunk::{Chunk, CHUNK_DIMENSIONSF, CHUNK_DIMENSIONS_USIZE},
        coordinates::{ChunkBlockCoordinate, CoordinateType},
        planet::Planet,
        Structure,
    },
    utils::array_utils::expand,
};
use rand::{seq::SliceRandom, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::{marker::PhantomData, mem::size_of};

use crate::{init::init_world::ServerSeed, state::GameState};

use super::{
    biome::{BiomeParameters, BiosphereBiomesRegistry},
    biosphere_generation_old::{BlockLayers, GenerateChunkFeaturesEvent},
    BiosphereMarkerComponent, BiosphereSeaLevel, TGenerateChunkEvent,
};

#[derive(Debug)]
pub(crate) struct NeedGeneratedChunk<T> {
    chunk: Chunk,
    structure_entity: Entity,
    chunk_pos: Vec3,
    structure_dimensions: CoordinateType,
    structure_location: Location,
    time: f32,
    _phantom: PhantomData<T>,
}

#[derive(Resource, Debug, Default)]
pub(crate) struct NeedGeneratedChunks<T>(Vec<NeedGeneratedChunk<T>>);

#[derive(Resource, Debug, Default)]
pub(crate) struct GeneratingChunks<T>(Vec<NeedGeneratedChunk<T>>);

pub(crate) fn send_and_read_chunks_gpu<T: BiosphereMarkerComponent, E: TGenerateChunkEvent>(
    mut needs_generated_chunks: ResMut<NeedGeneratedChunks<T>>,
    mut currently_generating_chunks: ResMut<GeneratingChunks<T>>,
    biosphere_biomes: Res<BiosphereBiomesRegistry<T>>,
    // biome_decider: Res<BiomeDecider<T>>,
    sea_level: Option<Res<BiosphereSeaLevel<T>>>,
    mut worker: ResMut<AppComputeWorker<BiosphereShaderWorker<T>>>,
    mut ev_writer: EventWriter<GenerateChunkFeaturesEvent<T>>,
    // blocks: Res<Registry<Block>>,
    mut q_structure: Query<&mut Structure>,

    time: Res<Time>,
) {
    if worker.ready() {
        if let Some(mut needs_generated_chunk) = currently_generating_chunks.0.pop() {
            if let Ok(mut structure) = q_structure.get_mut(needs_generated_chunk.structure_entity) {
                let v: Vec<f32> = worker.try_read_vec("values").expect("Failed to read values!");

                for (i, value) in v.into_iter().enumerate() {
                    let (x, y, z) = expand(i, CHUNK_DIMENSIONS_USIZE, CHUNK_DIMENSIONS_USIZE);

                    if value >= 0.0 {
                        let ideal_biome = biosphere_biomes.ideal_biome_for(BiomeParameters {
                            ideal_elevation: 50.0,
                            ideal_humidity: 50.0,
                            ideal_temperature: 50.0,
                        });

                        let block_layers: &BlockLayers = ideal_biome.block_layers();

                        let block = block_layers.block_for_depth(value as u64);

                        let coord = needs_generated_chunk.chunk_pos + Vec3::new(x as f32, y as f32, z as f32);

                        let face = Planet::planet_face_relative(coord);

                        needs_generated_chunk.chunk.set_block_at(
                            ChunkBlockCoordinate::new(x as CoordinateType, y as CoordinateType, z as CoordinateType),
                            &block,
                            face,
                        );
                    }
                }

                info!(
                    "Got generated chunk - took {}ms to generate",
                    1000.0 * (time.elapsed_seconds() - needs_generated_chunk.time)
                );

                // ideal_biome.generate_face_chunk(self_as_dyn, block_coords, s_dimensions, chunk, up, biome_id_list, self_biome_id, elevation, sea_level)

                ev_writer.send(GenerateChunkFeaturesEvent {
                    chunk_coords: needs_generated_chunk.chunk.chunk_coordinates(),
                    structure_entity: needs_generated_chunk.structure_entity,
                    _phantom: Default::default(),
                });

                structure.set_chunk(needs_generated_chunk.chunk);
            }
        } else {
            warn!("Something wacky happened");
        }
    }

    if currently_generating_chunks.0.is_empty() {
        if let Some(mut todo) = needs_generated_chunks.0.pop() {
            let structure_loc = todo.structure_location.absolute_coords_f32();

            let params = GenerationParams {
                chunk_coords: Vec4::new(todo.chunk_pos.x, todo.chunk_pos.y, todo.chunk_pos.z, 0.0),
                scale: Vec4::splat(1.0),
                sea_level: Vec4::splat((sea_level.map(|x| x.level).unwrap_or(0.75) * (todo.structure_dimensions / 2) as f32) as f32),
                structure_pos: Vec4::new(structure_loc.x, structure_loc.y, structure_loc.z, 0.0),
            };

            worker.write("params", &params);

            let vals: Vec<f32> = vec![0.0; CHUNK_DIMENSIONS_USIZE * CHUNK_DIMENSIONS_USIZE * CHUNK_DIMENSIONS_USIZE];

            worker.write_slice("values", &vals);

            todo.time = time.elapsed_seconds();

            currently_generating_chunks.0.push(todo);
            worker.execute();
        }
    }
}

/// Calls generate_face_chunk, generate_edge_chunk, and generate_corner_chunk to generate the chunks of a planet.
pub(crate) fn generate_planet<T: BiosphereMarkerComponent, E: TGenerateChunkEvent>(
    mut query: Query<(&mut Structure, &Location)>,
    mut events: EventReader<E>,

    mut needs_generated_chunks: ResMut<NeedGeneratedChunks<T>>,
) {
    let chunks = events
        .read()
        .filter_map(|ev| {
            let structure_entity = ev.get_structure_entity();
            let coords = ev.get_chunk_coordinates();

            if let Ok((mut structure, _)) = query.get_mut(structure_entity) {
                let Structure::Dynamic(planet) = structure.as_mut() else {
                    panic!("A planet must be dynamic!");
                };
                Some((structure_entity, planet.take_or_create_chunk_for_loading(coords)))
            } else {
                None
            }
        })
        .collect::<Vec<(Entity, Chunk)>>();

    // let thread_pool = AsyncComputeTaskPool::get();

    needs_generated_chunks
        .0
        .extend(chunks.into_iter().flat_map(|(structure_entity, chunk)| {
            let Ok((structure, location)) = query.get(structure_entity) else {
                return None;
            };

            let Structure::Dynamic(planet) = structure else {
                panic!("A planet must be dynamic!");
            };

            let s_dimensions = planet.block_dimensions();
            let location = *location;

            // This should be negative-most position of chunk, but chunk_relative_position returns the middle coordinate.
            let chunk_rel_pos = planet.chunk_relative_position(chunk.chunk_coordinates()) - Vec3::splat(CHUNK_DIMENSIONSF / 2.0);

            Some(NeedGeneratedChunk {
                chunk,
                chunk_pos: chunk_rel_pos,
                structure_dimensions: s_dimensions,
                structure_entity,
                structure_location: location,
                time: 0.0,
                _phantom: Default::default(),
            })
        }));
}

#[derive(Debug, ShaderType, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
struct GenerationParams {
    // Everythihng has to be a vec4 because padding. Otherwise things get super wack
    chunk_coords: Vec4,
    structure_pos: Vec4,
    sea_level: Vec4,
    scale: Vec4,
}

#[derive(TypePath, Default)]
struct ComputeShaderInstance<T: BiosphereMarkerComponent + TypePath>(PhantomData<T>);

impl<T: BiosphereMarkerComponent + TypePath> ComputeShader for ComputeShaderInstance<T> {
    fn shader() -> ShaderRef {
        "cosmos/shaders/compute.wgsl".into()
    }
}

// If you change this, make sure to modify the '32' values in the shader aswell.
const SIZE: u32 = 32;
// If you change this, make sure to modify the '512' values in the shader aswell.
const WORKGROUP_SIZE: u32 = 1024;

#[derive(Default)]
pub(crate) struct BiosphereShaderWorker<T: BiosphereMarkerComponent>(PhantomData<T>);

#[repr(C)]
#[derive(Debug, ShaderType, Pod, Zeroable, Clone, Copy)]
struct U32Vec4 {
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub w: u32,
}

impl U32Vec4 {
    pub fn new(x: u32, y: u32, z: u32, w: u32) -> Self {
        Self { x, y, z, w }
    }
}

impl<T: BiosphereMarkerComponent + TypePath> ComputeWorker for BiosphereShaderWorker<T> {
    fn build(world: &mut bevy::prelude::World) -> AppComputeWorker<Self> {
        const DIMS: usize = (SIZE * SIZE * SIZE) as usize;

        // let noise = noise::OpenSimplex::new(1596);
        // noise.
        // let perm_table = PermutationTable;

        let params = GenerationParams {
            chunk_coords: Vec4::splat(13.0),
            structure_pos: Vec4::splat(12.0),
            sea_level: Vec4::splat(11.0),
            scale: Vec4::splat(10.0),
        };

        let icrs = vec![1.0; DIMS];

        assert!((SIZE * SIZE * SIZE) % WORKGROUP_SIZE == 0);

        // let permutation_table: [U32Vec4; 256 / 4] = [
        //     // U32Vec4::new(151, 160, 137, 91),
        //     U32Vec4::new(151, 160, 137, 91),
        //     U32Vec4::new(90, 15, 131, 13),
        //     U32Vec4::new(201, 95, 96, 53),
        //     U32Vec4::new(194, 233, 7, 225),
        //     U32Vec4::new(140, 36, 103, 30),
        //     U32Vec4::new(69, 142, 8, 99),
        //     U32Vec4::new(37, 240, 21, 10),
        //     U32Vec4::new(23, 190, 6, 148),
        //     U32Vec4::new(247, 120, 234, 75),
        //     U32Vec4::new(0, 26, 197, 62),
        //     U32Vec4::new(94, 252, 219, 203),
        //     U32Vec4::new(117, 35, 11, 32),
        //     U32Vec4::new(57, 177, 33, 88),
        //     U32Vec4::new(237, 149, 56, 87),
        //     U32Vec4::new(174, 20, 125, 136),
        //     U32Vec4::new(171, 168, 68, 175),
        //     U32Vec4::new(74, 165, 71, 134),
        //     U32Vec4::new(139, 48, 27, 166),
        //     U32Vec4::new(77, 146, 158, 231),
        //     U32Vec4::new(83, 111, 229, 122),
        //     U32Vec4::new(60, 211, 133, 230),
        //     U32Vec4::new(220, 105, 92, 41),
        //     U32Vec4::new(55, 46, 245, 40),
        //     U32Vec4::new(244, 102, 143, 54),
        //     U32Vec4::new(65, 25, 63, 161),
        //     U32Vec4::new(1, 216, 80, 73),
        //     U32Vec4::new(209, 76, 132, 187),
        //     U32Vec4::new(208, 89, 18, 169),
        //     U32Vec4::new(200, 196, 135, 130),
        //     U32Vec4::new(116, 188, 159, 86),
        //     U32Vec4::new(164, 100, 109, 198),
        //     U32Vec4::new(173, 186, 3, 64),
        //     U32Vec4::new(52, 217, 226, 250),
        //     U32Vec4::new(124, 123, 5, 202),
        //     U32Vec4::new(38, 147, 118, 126),
        //     U32Vec4::new(255, 82, 85, 212),
        //     U32Vec4::new(207, 206, 59, 227),
        //     U32Vec4::new(47, 16, 58, 17),
        //     U32Vec4::new(182, 189, 28, 42),
        //     U32Vec4::new(223, 183, 170, 213),
        //     U32Vec4::new(119, 248, 152, 2),
        //     U32Vec4::new(44, 154, 163, 70),
        //     U32Vec4::new(221, 153, 101, 155),
        //     U32Vec4::new(167, 43, 172, 9),
        //     U32Vec4::new(129, 22, 39, 253),
        //     U32Vec4::new(19, 98, 108, 110),
        //     U32Vec4::new(79, 113, 224, 232),
        //     U32Vec4::new(178, 185, 112, 104),
        //     U32Vec4::new(218, 246, 97, 228),
        //     U32Vec4::new(251, 34, 242, 193),
        //     U32Vec4::new(238, 210, 144, 12),
        //     U32Vec4::new(191, 179, 162, 241),
        //     U32Vec4::new(81, 51, 145, 235),
        //     U32Vec4::new(249, 14, 239, 107),
        //     U32Vec4::new(49, 192, 214, 31),
        //     U32Vec4::new(181, 199, 106, 157),
        //     U32Vec4::new(184, 84, 204, 176),
        //     U32Vec4::new(115, 121, 50, 45),
        //     U32Vec4::new(127, 4, 150, 254),
        //     U32Vec4::new(138, 236, 205, 93),
        //     U32Vec4::new(222, 114, 67, 29),
        //     U32Vec4::new(24, 72, 243, 141),
        //     U32Vec4::new(128, 195, 78, 66),
        //     U32Vec4::new(215, 61, 156, 180),
        // ];

        let worker = AppComputeWorkerBuilder::new(world)
            .one_shot()
            // .add_empty_uniform("params", std::mem::size_of::<GenerationParams>() as u64) // GenerationParams
            .add_empty_uniform("permutation_table", size_of::<[U32Vec4; 256 / 4]>() as u64) // Vec<f32>
            .add_uniform("params", &params) // GenerationParams
            .add_staging("values", &icrs)
            .add_pass::<ComputeShaderInstance<T>>(
                [SIZE * SIZE * SIZE / WORKGROUP_SIZE, 1, 1], //SIZE / WORKGROUP_SIZE, SIZE / WORKGROUP_SIZE, SIZE / WORKGROUP_SIZE
                &["permutation_table", "params", "values"],
            )
            .build();

        worker
    }
}

#[derive(Resource)]
struct PermutationTable(Vec<U32Vec4>);

fn set_permutation_table<T: BiosphereMarkerComponent>(
    perm_table: Res<PermutationTable>,
    mut worker: ResMut<AppComputeWorker<BiosphereShaderWorker<T>>>,
) {
    worker.write_slice("permutation_table", &perm_table.0);
}

fn setup_permutation_table(seed: Res<ServerSeed>, mut commands: Commands) {
    let seed = seed.as_u64();
    let mut permutation_table_array: Vec<u8> = (0..256).map(|x| x as u8).collect();

    let mut real = [0; 32];
    real[0] = 1;
    for i in 1..4 {
        real[i * 4] = seed as u8;
        real[(i * 4) + 1] = (seed >> 8) as u8;
        real[(i * 4) + 2] = (seed >> 16) as u8;
        real[(i * 4) + 3] = (seed >> 24) as u8;
        real[(i * 4) + 4] = (seed >> 32) as u8;
        real[(i * 4) + 5] = (seed >> 40) as u8;
        real[(i * 4) + 6] = (seed >> 48) as u8;
        real[(i * 4) + 7] = (seed >> 56) as u8;
    }

    let mut rng = ChaCha8Rng::from_seed(real);

    permutation_table_array.shuffle(&mut rng);

    // Convert it to more wgpu friendly table

    let permutation_table: Vec<U32Vec4> = permutation_table_array
        .into_iter()
        .array_chunks::<4>()
        .map(|[x, y, z, w]| U32Vec4::new(x as u32, y as u32, z as u32, w as u32))
        .collect();

    commands.insert_resource(PermutationTable(permutation_table));

    // let permutation_table: [U32Vec4; 256 / 4] = [
    //     U32Vec4::new(151, 160, 137, 91),
    //     U32Vec4::new(90, 15, 131, 13),
    //     U32Vec4::new(201, 95, 96, 53),
    //     U32Vec4::new(194, 233, 7, 225),
    //     U32Vec4::new(140, 36, 103, 30),
    //     U32Vec4::new(69, 142, 8, 99),
    //     U32Vec4::new(37, 240, 21, 10),
    //     U32Vec4::new(23, 190, 6, 148),
    //     U32Vec4::new(247, 120, 234, 75),
    //     U32Vec4::new(0, 26, 197, 62),
    //     U32Vec4::new(94, 252, 219, 203),
    //     U32Vec4::new(117, 35, 11, 32),
    //     U32Vec4::new(57, 177, 33, 88),
    //     U32Vec4::new(237, 149, 56, 87),
    //     U32Vec4::new(174, 20, 125, 136),
    //     U32Vec4::new(171, 168, 68, 175),
    //     U32Vec4::new(74, 165, 71, 134),
    //     U32Vec4::new(139, 48, 27, 166),
    //     U32Vec4::new(77, 146, 158, 231),
    //     U32Vec4::new(83, 111, 229, 122),
    //     U32Vec4::new(60, 211, 133, 230),
    //     U32Vec4::new(220, 105, 92, 41),
    //     U32Vec4::new(55, 46, 245, 40),
    //     U32Vec4::new(244, 102, 143, 54),
    //     U32Vec4::new(65, 25, 63, 161),
    //     U32Vec4::new(1, 216, 80, 73),
    //     U32Vec4::new(209, 76, 132, 187),
    //     U32Vec4::new(208, 89, 18, 169),
    //     U32Vec4::new(200, 196, 135, 130),
    //     U32Vec4::new(116, 188, 159, 86),
    //     U32Vec4::new(164, 100, 109, 198),
    //     U32Vec4::new(173, 186, 3, 64),
    //     U32Vec4::new(52, 217, 226, 250),
    //     U32Vec4::new(124, 123, 5, 202),
    //     U32Vec4::new(38, 147, 118, 126),
    //     U32Vec4::new(255, 82, 85, 212),
    //     U32Vec4::new(207, 206, 59, 227),
    //     U32Vec4::new(47, 16, 58, 17),
    //     U32Vec4::new(182, 189, 28, 42),
    //     U32Vec4::new(223, 183, 170, 213),
    //     U32Vec4::new(119, 248, 152, 2),
    //     U32Vec4::new(44, 154, 163, 70),
    //     U32Vec4::new(221, 153, 101, 155),
    //     U32Vec4::new(167, 43, 172, 9),
    //     U32Vec4::new(129, 22, 39, 253),
    //     U32Vec4::new(19, 98, 108, 110),
    //     U32Vec4::new(79, 113, 224, 232),
    //     U32Vec4::new(178, 185, 112, 104),
    //     U32Vec4::new(218, 246, 97, 228),
    //     U32Vec4::new(251, 34, 242, 193),
    //     U32Vec4::new(238, 210, 144, 12),
    //     U32Vec4::new(191, 179, 162, 241),
    //     U32Vec4::new(81, 51, 145, 235),
    //     U32Vec4::new(249, 14, 239, 107),
    //     U32Vec4::new(49, 192, 214, 31),
    //     U32Vec4::new(181, 199, 106, 157),
    //     U32Vec4::new(184, 84, 204, 176),
    //     U32Vec4::new(115, 121, 50, 45),
    //     U32Vec4::new(127, 4, 150, 254),
    //     U32Vec4::new(138, 236, 205, 93),
    //     U32Vec4::new(222, 114, 67, 29),
    //     U32Vec4::new(24, 72, 243, 141),
    //     U32Vec4::new(128, 195, 78, 66),
    //     U32Vec4::new(215, 61, 156, 180),
    // ];
}

pub(super) fn register_biosphere<T: BiosphereMarkerComponent>(app: &mut App) {
    app.add_plugins(AppComputeWorkerPlugin::<BiosphereShaderWorker<T>>::default())
        .add_systems(OnEnter(GameState::PostLoading), set_permutation_table::<T>)
        .init_resource::<NeedGeneratedChunks<T>>()
        .init_resource::<GeneratingChunks<T>>();
}

pub(super) fn register(app: &mut App) {
    app.add_systems(OnEnter(GameState::PreLoading), setup_permutation_table);
}
