use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::render_resource::*,
};
use cosmos_core::structure::{planet::Planet, Structure};

fn on_spawn_planet(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut commands: Commands,
    query: Query<(Entity, &Structure), Added<Planet>>,
) {
    for (entity, planet) in query.iter() {
        commands.entity(entity).with_children(|p| {
            p.spawn((
                MaterialMeshBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube {
                        size: planet.block_dimensions().x as f32 * 1.1,
                    })),
                    transform: Transform::from_xyz(0.0, 0.0, 0.0),
                    material: materials.add(CustomMaterial {}),
                    ..default()
                },
                NotShadowCaster,
                NotShadowReceiver,
                Name::new("Atmosphere Shader"),
            ));
        });
    }
}

#[derive(AsBindGroup, TypeUuid, TypePath, Debug, Clone)]
#[uuid = "a3d71c04-d054-4946-80f8-ba6cfbc90cad"]
struct CustomMaterial {}

impl Material for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        "cosmos/shaders/planet_atmosphere.wgsl".into()
    }
}

pub(super) fn register(app: &mut App) {
    app.add_plugins(MaterialPlugin::<CustomMaterial>::default())
        .add_systems(Update, on_spawn_planet);
}
