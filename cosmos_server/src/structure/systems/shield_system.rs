//! Represents all the energy stored on a structure

use bevy::{
    core::Name,
    ecs::query::Added,
    hierarchy::BuildChildren,
    log::warn,
    math::Vec3,
    prelude::{in_state, App, Commands, EventReader, IntoSystemConfigs, OnEnter, Query, Res, ResMut, Update},
    transform::{
        components::{GlobalTransform, Transform},
        TransformBundle,
    },
};

use bevy_rapier3d::geometry::{Collider, ColliderMassProperties, CollisionGroups, Group, Sensor};
use cosmos_core::{
    block::Block,
    events::block_events::BlockChangedEvent,
    persistence::LoadingDistance,
    physics::location::Location,
    registry::{identifiable::Identifiable, Registry},
    structure::{
        events::StructureLoadedEvent,
        loading::StructureLoadingSet,
        shields::Shield,
        systems::{
            shield_system::{ShieldBlocks, ShieldProperty, ShieldSystem},
            StructureSystem, StructureSystemType, StructureSystems,
        },
        Structure,
    },
};

use crate::state::GameState;

use super::sync::register_structure_system;

fn register_energy_blocks(blocks: Res<Registry<Block>>, mut storage: ResMut<ShieldBlocks>) {
    if let Some(block) = blocks.from_id("cosmos:shield") {
        storage.0.insert(
            block.id(),
            ShieldProperty {
                shield_range_increase: 1.0,
                shield_strength: 1.0,
            },
        );
    }
}

fn block_update_system(
    mut event: EventReader<BlockChangedEvent>,
    energy_storage_blocks: Res<ShieldBlocks>,
    mut system_query: Query<&mut ShieldSystem>,
    systems_query: Query<&StructureSystems>,
) {
    for ev in event.read() {
        if let Ok(systems) = systems_query.get(ev.structure_entity) {
            if let Ok(mut system) = systems.query_mut(&mut system_query) {
                if let Some(&prop) = energy_storage_blocks.0.get(&ev.old_block) {
                    system.block_removed(prop, ev.block.coords());
                }

                if let Some(&prop) = energy_storage_blocks.0.get(&ev.new_block) {
                    system.block_added(prop, ev.block.coords());
                }
            }
        }
    }
}

fn structure_loaded_event(
    mut event_reader: EventReader<StructureLoadedEvent>,
    mut structure_query: Query<(&Structure, &mut StructureSystems)>,
    mut commands: Commands,
    energy_storage_blocks: Res<ShieldBlocks>,
    registry: Res<Registry<StructureSystemType>>,
) {
    for ev in event_reader.read() {
        if let Ok((structure, mut systems)) = structure_query.get_mut(ev.structure_entity) {
            let mut system = ShieldSystem::default();

            for block in structure.all_blocks_iter(false) {
                if let Some(&prop) = energy_storage_blocks.0.get(&structure.block_id_at(block.coords())) {
                    system.block_added(prop, block.coords());
                }
            }

            systems.add_system(&mut commands, system, &registry);
        }
    }
}

pub const SHIELD_COLLISION_GROUP: Group = Group::GROUP_3;

fn add_shield(
    mut commands: Commands,
    q_added_ship: Query<&StructureSystem, Added<ShieldSystem>>,
    q_loc: Query<(&Location, &GlobalTransform)>,
) {
    for ss in q_added_ship.iter() {
        let structure_entity = ss.structure_entity();
        let Ok((loc, g_trans)) = q_loc.get(structure_entity) else {
            warn!("No loc/g-trans");
            continue;
        };

        let shield_pos = Vec3::ZERO;
        // Locations don't account for parent rotation
        let shield_loc = g_trans.affine().matrix3.mul_vec3(shield_pos);

        commands.entity(structure_entity).with_children(|p| {
            p.spawn((
                Name::new("Shield"),
                TransformBundle::from_transform(Transform::from_translation(shield_pos)),
                *loc + shield_loc,
                LoadingDistance::new(1, 2),
                Collider::ball(20.0),
                CollisionGroups::new(SHIELD_COLLISION_GROUP, SHIELD_COLLISION_GROUP),
                ColliderMassProperties::Mass(0.0),
                Sensor,
                Shield {
                    max_strength: 100.0,
                    radius: 20.0,
                    strength: 1.0,
                },
            ));
        });
    }
}

pub(super) fn register(app: &mut App) {
    app.insert_resource(ShieldBlocks::default())
        .add_systems(OnEnter(GameState::PostLoading), register_energy_blocks)
        .add_systems(
            Update,
            (
                add_shield, // before so this runs next frame (so the globaltransform has been added to the structure)
                structure_loaded_event.in_set(StructureLoadingSet::StructureLoaded),
                block_update_system,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        )
        .register_type::<ShieldSystem>();

    register_structure_system::<ShieldSystem>(app, false, "cosmos:shield");
}
