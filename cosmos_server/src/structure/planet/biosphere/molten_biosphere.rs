//! Creates a molten planet

use bevy::prelude::{
    in_state, App, Commands, Component, Entity, Event, EventReader, EventWriter, IntoSystemConfigs, OnEnter, Query, Res, Update,
};
use cosmos_core::{
    block::{Block, BlockFace},
    events::block_events::BlockChangedEvent,
    physics::location::Location,
    registry::Registry,
    structure::{chunk::CHUNK_DIMENSIONS, planet::Planet, rotate, ChunkInitEvent, Structure},
};

use crate::{init::init_world::ServerSeed, GameState};

use super::{
    biosphere_generation::{
        generate_planet, notify_when_done_generating_terrain, BlockRanges, DefaultBiosphereGenerationStrategy, GenerateChunkFeaturesEvent,
        GenerationParemeters,
    },
    register_biosphere, TBiosphere, TGenerateChunkEvent, TemperatureRange,
};

#[derive(Component, Debug, Default, Clone)]
/// Marks that this is for a grass biosphere
pub struct MoltenBiosphereMarker;

/// Marks that a grass chunk needs generated
#[derive(Debug, Event)]
pub struct MoltenChunkNeedsGeneratedEvent {
    x: usize,
    y: usize,
    z: usize,
    structure_entity: Entity,
}

impl TGenerateChunkEvent for MoltenChunkNeedsGeneratedEvent {
    fn new(x: usize, y: usize, z: usize, structure_entity: Entity) -> Self {
        Self { x, y, z, structure_entity }
    }

    fn get_structure_entity(&self) -> Entity {
        self.structure_entity
    }

    fn get_chunk_coordinates(&self) -> (usize, usize, usize) {
        (self.x, self.y, self.z)
    }
}

#[derive(Default, Debug)]
/// Creates a molten planet
pub struct MoltenBiosphere;

impl TBiosphere<MoltenBiosphereMarker, MoltenChunkNeedsGeneratedEvent> for MoltenBiosphere {
    fn get_marker_component(&self) -> MoltenBiosphereMarker {
        MoltenBiosphereMarker {}
    }

    fn get_generate_chunk_event(&self, x: usize, y: usize, z: usize, structure_entity: Entity) -> MoltenChunkNeedsGeneratedEvent {
        MoltenChunkNeedsGeneratedEvent::new(x, y, z, structure_entity)
    }
}

fn make_block_ranges(block_registry: Res<Registry<Block>>, mut commands: Commands) {
    commands.insert_resource(
        BlockRanges::<MoltenBiosphereMarker>::default()
            .with_sea_level_block("cosmos:cheese", &block_registry, -20)
            .expect("Cheese missing!")
            .with_range("cosmos:molten_stone", &block_registry, 0)
            .expect("Molten Stone missing"),
    );
}

// Fills the chunk at the given coordinates with spikes
fn generate_spikes(
    (cx, cy, cz): (usize, usize, usize),
    structure: &mut Structure,
    location: &Location,
    block_event_writer: &mut EventWriter<BlockChangedEvent>,
    blocks: &Registry<Block>,
    seed: ServerSeed,
) {
    let (sx, sy, sz) = (cx * CHUNK_DIMENSIONS, cy * CHUNK_DIMENSIONS, cz * CHUNK_DIMENSIONS);
    let s_dimension = structure.blocks_height();

    let molten_stone = blocks.from_id("cosmos:molten_stone").expect("Missing molten_stone");

    let structure_coords = location.absolute_coords_f64();

    let faces = Planet::chunk_planet_faces((sx, sy, sz), s_dimension);
    for block_up in faces.iter() {
        // Getting the noise value for every block in the chunk, to find where to put trees.
        let noise_height = match block_up {
            BlockFace::Front | BlockFace::Top | BlockFace::Right => structure.blocks_height(),
            _ => 0,
        };

        for z in 0..CHUNK_DIMENSIONS {
            for x in 0..CHUNK_DIMENSIONS {
                let (nx, ny, nz) = match block_up {
                    BlockFace::Front | BlockFace::Back => ((sx + x) as f64, (sy + z) as f64, noise_height as f64),
                    BlockFace::Top | BlockFace::Bottom => ((sx + x) as f64, noise_height as f64, (sz + z) as f64),
                    BlockFace::Right | BlockFace::Left => (noise_height as f64, (sy + x) as f64, (sz + z) as f64),
                };

                let rng = seed
                    .chaos_hash(nx + structure_coords.x, ny + structure_coords.y, nz + structure_coords.z)
                    .abs()
                    % 20;

                if rng == 0 {
                    let rng = seed
                        .chaos_hash(
                            2000.0 + nx + structure_coords.x,
                            2000.0 + ny + structure_coords.y,
                            2000.0 + nz + structure_coords.z,
                        )
                        .abs()
                        % 4;

                    let (bx, by, bz) = match block_up {
                        BlockFace::Front | BlockFace::Back => (sx + x, sy + z, sz),
                        BlockFace::Top | BlockFace::Bottom => (sx + x, sy, sz + z),
                        BlockFace::Right | BlockFace::Left => (sx, sy + x, sz + z),
                    };

                    let s_dimensions = (s_dimension, s_dimension, s_dimension);

                    if let Ok(start_checking) = rotate((bx, by, bz), (0, CHUNK_DIMENSIONS as i32 - 1, 0), s_dimensions, block_up) {
                        'spike_placement: for dy_down in 0..CHUNK_DIMENSIONS {
                            if let Ok(rotated) = rotate(start_checking, (0, -(dy_down as i32), 0), s_dimensions, block_up) {
                                if structure.block_at_tuple(rotated, blocks) == molten_stone {
                                    for dy in 1..=rng {
                                        if let Ok(rel_pos) =
                                            rotate(start_checking, (0, dy as i32 - dy_down as i32, 0), s_dimensions, block_up)
                                        {
                                            structure.set_block_at_tuple(rel_pos, molten_stone, block_up, blocks, Some(block_event_writer));
                                        }
                                    }
                                    break 'spike_placement;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Sends a ChunkInitEvent for every chunk that's done generating, monitors when chunks are finished generating, makes trees.
pub fn generate_chunk_features(
    mut event_reader: EventReader<GenerateChunkFeaturesEvent<MoltenBiosphereMarker>>,
    mut init_event_writer: EventWriter<ChunkInitEvent>,
    mut block_event_writer: EventWriter<BlockChangedEvent>,
    mut structure_query: Query<(&mut Structure, &Location)>,
    blocks: Res<Registry<Block>>,
    seed: Res<ServerSeed>,
) {
    for ev in event_reader.iter() {
        if let Ok((mut structure, location)) = structure_query.get_mut(ev.structure_entity) {
            let (cx, cy, cz) = ev.chunk_coords;

            generate_spikes((cx, cy, cz), &mut structure, location, &mut block_event_writer, &blocks, *seed);

            init_event_writer.send(ChunkInitEvent {
                structure_entity: ev.structure_entity,
                x: cx,
                y: cy,
                z: cz,
            });
        }
    }
}

pub(super) fn register(app: &mut App) {
    register_biosphere::<MoltenBiosphereMarker, MoltenChunkNeedsGeneratedEvent>(
        app,
        "cosmos:biosphere_molten",
        TemperatureRange::new(450.0, 1000000000.0),
    );

    app.add_systems(
        Update,
        (
            generate_planet::<MoltenBiosphereMarker, MoltenChunkNeedsGeneratedEvent, DefaultBiosphereGenerationStrategy>,
            notify_when_done_generating_terrain::<MoltenBiosphereMarker>,
            generate_chunk_features,
        )
            .run_if(in_state(GameState::Playing)),
    )
    .insert_resource(GenerationParemeters::<MoltenBiosphereMarker>::new(0.10, 7.0, 9));

    app.add_system(make_block_ranges.in_schedule(OnEnter(GameState::PostLoading)));
}
