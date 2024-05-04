//! Represents all the energy stored on a structure

use bevy::{
    ecs::{
        change_detection::DetectChanges,
        component::Component,
        entity::Entity,
        query::{With, Without},
        world::Ref,
    },
    math::{Quat, Vec3},
    prelude::{in_state, App, Commands, EventReader, IntoSystemConfigs, Query, Res, Update},
    transform::components::GlobalTransform,
};

use bevy_rapier3d::{
    dynamics::{FixedJointBuilder, ImpulseJoint, PhysicsWorld, Velocity},
    geometry::{CollisionGroups, Group},
    pipeline::QueryFilter,
    plugin::RapierContext,
};
use cosmos_core::{
    block::Block,
    events::block_events::BlockChangedEvent,
    physics::structure_physics::ChunkPhysicsPart,
    registry::{identifiable::Identifiable, Registry},
    structure::{
        events::StructureLoadedEvent,
        loading::StructureLoadingSet,
        shields::SHIELD_COLLISION_GROUP,
        systems::{dock_system::DockSystem, StructureSystem, StructureSystemType, StructureSystems, SystemActive},
        Structure,
    },
    utils::quat_math::QuatMath,
};

use crate::state::GameState;

use super::sync::register_structure_system;

const MAX_DOCK_CHECK: f32 = 1.3;

fn dock_block_update_system(
    mut event: EventReader<BlockChangedEvent>,
    blocks: Res<Registry<Block>>,
    mut system_query: Query<&mut DockSystem>,
    q_systems: Query<&StructureSystems>,
) {
    for ev in event.read() {
        let Ok(systems) = q_systems.get(ev.structure_entity) else {
            continue;
        };

        let Ok(mut system) = systems.query_mut(&mut system_query) else {
            continue;
        };

        if blocks.from_numeric_id(ev.old_block).unlocalized_name() == "cosmos:ship_dock" {
            system.block_removed(ev.block.coords());
        }

        if blocks.from_numeric_id(ev.new_block).unlocalized_name() == "cosmos:ship_dock" {
            system.block_added(ev.block.coords());
        }
    }
}

fn dock_structure_loaded_event_processor(
    mut event_reader: EventReader<StructureLoadedEvent>,
    mut structure_query: Query<(&Structure, &mut StructureSystems)>,
    blocks: Res<Registry<Block>>,
    mut commands: Commands,
    registry: Res<Registry<StructureSystemType>>,
) {
    for ev in event_reader.read() {
        if let Ok((structure, mut systems)) = structure_query.get_mut(ev.structure_entity) {
            let mut system = DockSystem::default();

            for block in structure.all_blocks_iter(false) {
                if block.block(structure, &blocks).unlocalized_name() == "cosmos:ship_dock" {
                    system.block_added(block.coords());
                }
            }

            systems.add_system(&mut commands, system, &registry);
        }
    }
}

#[derive(Component)]
struct Docked(Entity);

#[derive(Component)]
struct JustUndocked;

fn on_active(
    context: Res<RapierContext>,
    q_docked: Query<&Docked>,
    q_structure: Query<(&Structure, &GlobalTransform, &PhysicsWorld)>,
    q_active: Query<(Entity, &StructureSystem, &DockSystem, Ref<SystemActive>, Option<&JustUndocked>)>,
    q_inactive: Query<Entity, (With<DockSystem>, Without<SystemActive>, With<JustUndocked>)>,
    q_chunk_entity: Query<&ChunkPhysicsPart>,
    blocks: Res<Registry<Block>>,
    q_velocity: Query<&Velocity>,
    mut commands: Commands,
) {
    for e in q_inactive.iter() {
        commands.entity(e).remove::<JustUndocked>();
    }

    for (system_entity, ss, ds, active_system_flag, just_undocked) in q_active.iter() {
        let Ok((structure, g_trans, pw)) = q_structure.get(ss.structure_entity()) else {
            continue;
        };

        let docked = q_docked.get(ss.structure_entity());

        if active_system_flag.is_added() {
            if let Ok(docked) = docked {
                let vel = q_velocity.get(docked.0).copied().unwrap_or_default();

                commands
                    .entity(ss.structure_entity())
                    .remove::<Docked>()
                    .remove::<ImpulseJoint>()
                    .insert(vel);

                commands.entity(system_entity).insert(JustUndocked);

                continue;
            }
        }

        if docked.is_ok() || just_undocked.is_some() {
            continue;
        }

        for &docking_block in ds.block_locations() {
            let rel_pos = structure.block_relative_position(docking_block);
            let block_rotation = structure.block_rotation(docking_block);
            let docking_look_face = block_rotation.local_front();
            let front_direction = docking_look_face.direction_vec3();

            let abs_block_pos = g_trans.transform_point(rel_pos);

            let my_rotation = Quat::from_affine3(&g_trans.affine());
            let ray_dir = my_rotation.mul_vec3(front_direction);

            let Ok(Some((entity, intersection))) = context.cast_ray_and_get_normal(
                pw.world_id,
                abs_block_pos,
                ray_dir,
                MAX_DOCK_CHECK,
                false,
                QueryFilter::new()
                    .groups(CollisionGroups::new(
                        Group::ALL & !SHIELD_COLLISION_GROUP,
                        Group::ALL & !SHIELD_COLLISION_GROUP,
                    ))
                    .predicate(&|e| {
                        let Ok(ce) = q_chunk_entity.get(e) else {
                            return false;
                        };

                        ce.structure_entity != ss.structure_entity()
                    }),
            ) else {
                continue;
            };

            let Ok(structure_entity) = q_chunk_entity.get(entity).map(|x| x.structure_entity) else {
                continue;
            };

            let Ok((hit_structure, hit_g_trans, _)) = q_structure.get(structure_entity) else {
                return;
            };

            let moved_point = intersection.point - intersection.normal * 0.01;

            let point = hit_g_trans.compute_matrix().inverse().transform_point3(moved_point);

            let Ok(hit_coords) = hit_structure.relative_coords_to_local_coords_checked(point.x, point.y, point.z) else {
                return;
            };

            let block: &Block = hit_structure.block_at(hit_coords, &blocks);

            if block.unlocalized_name() != "cosmos:ship_dock" {
                continue;
            };

            let hit_block_face = hit_structure.block_rotation(hit_coords).local_front();
            let hit_rotation = Quat::from_affine3(&hit_g_trans.affine());
            let front_direction = hit_rotation.mul_vec3(hit_block_face.direction_vec3());

            let dotted = ray_dir.dot(front_direction);

            if dotted > -0.92 {
                continue;
            }

            // let q = Transform::from_translation(Vec3::ZERO)
            //     .looking_to(-hit_block_face.direction_vec3(), Vec3::Y)
            //     .rotation;

            // let q = match hit_block_face {
            //     cosmos_core::block::BlockFace::Right => fun_name(my_rotation, Vec3::X, Quat::from_axis_angle(Vec3::X, -PI / 2.0)),
            //     cosmos_core::block::BlockFace::Left => fun_name(my_rotation, Vec3::X, Quat::from_axis_angle(Vec3::X, PI / 2.0)),
            //     cosmos_core::block::BlockFace::Top => fun_name(my_rotation, Vec3::Y, Quat::IDENTITY),
            //     cosmos_core::block::BlockFace::Bottom => fun_name(my_rotation, Vec3::Y, Quat::from_axis_angle(Vec3::Z, PI)),
            //     cosmos_core::block::BlockFace::Front => fun_name(my_rotation, Vec3::Z, Quat::from_axis_angle(Vec3::X, -PI / 2.0)),
            //     cosmos_core::block::BlockFace::Back => fun_name(my_rotation, Vec3::Z, Quat::from_axis_angle(Vec3::X, PI / 2.0)),
            // };

            let relative_docked_ship_rotation = snap_to_right_angle(hit_rotation.inverse() * my_rotation);

            let rel_pos = hit_structure.block_relative_position(hit_coords)
                - relative_docked_ship_rotation.mul_vec3(structure.block_relative_position(docking_block) - Vec3::new(0.0, 0.0, 1.0));

            let joint = FixedJointBuilder::default()
                .local_anchor1(rel_pos)
                .local_basis1(relative_docked_ship_rotation);

            // dock
            commands
                .entity(ss.structure_entity())
                .insert(Docked(structure_entity))
                .insert(ImpulseJoint::new(structure_entity, joint));
        }
    }
}

/// Takes a rotation and returns the rotation that is the closest with all axes pointing at right angle intervals
fn snap_to_right_angle(rot: Quat) -> Quat {
    let nearest_forward = nearest_axis(rot * Vec3::Z);
    let nearest_up = nearest_axis(rot * Vec3::Y);
    // return Quat::look_to(nearest_forward, nearest_up);
    return Quat::looking_to(-nearest_forward, nearest_up);
}

/// Find the absolute axis that is closest to the given direction
fn nearest_axis(direction: Vec3) -> Vec3 {
    let x = direction.x.abs();
    let y = direction.y.abs();
    let z = direction.z.abs();
    if x > y && x > z {
        Vec3::new(direction.x.signum(), 0.0, 0.0)
    } else if y > x && y > z {
        Vec3::new(0.0, direction.y.signum(), 0.0)
    } else {
        Vec3::new(0.0, 0.0, direction.z.signum())
    }
}

pub(super) fn register(app: &mut App) {
    app.add_systems(
        Update,
        (
            dock_structure_loaded_event_processor.in_set(StructureLoadingSet::StructureLoaded),
            dock_block_update_system,
            on_active,
        )
            .run_if(in_state(GameState::Playing)),
    );

    register_structure_system::<DockSystem>(app, true, "cosmos:ship_dock");
}
