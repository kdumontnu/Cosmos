use bevy::{
    ecs::system::EntityCommands,
    prelude::{BuildChildren, PbrBundle, Transform},
};
use bevy_rapier3d::prelude::{RigidBody, Velocity};

use crate::{physics::structure_physics::StructurePhysics, structure::structure::Structure};

pub trait TStructureBuilder {
    fn insert_structure(
        &self,
        entity: &mut EntityCommands,
        transform: Transform,
        velocity: Velocity,
        structure: &mut Structure,
    );
}
#[derive(Default)]
pub struct StructureBuilder {}

impl TStructureBuilder for StructureBuilder {
    fn insert_structure(
        &self,
        entity: &mut EntityCommands,
        transform: Transform,
        velocity: Velocity,
        structure: &mut Structure,
    ) {
        let physics_updater = StructurePhysics::new(structure, entity.id());

        entity
            .insert_bundle(PbrBundle {
                transform,
                ..Default::default()
            })
            .insert(velocity)
            .with_children(|parent| {
                for z in 0..structure.length() {
                    for y in 0..structure.height() {
                        for x in 0..structure.width() {
                            let entity = parent
                                .spawn()
                                .insert_bundle(PbrBundle {
                                    transform: Transform::from_translation(
                                        structure.chunk_relative_position(x, y, z).into(),
                                    ),
                                    ..Default::default()
                                })
                                .id();

                            structure.set_chunk_entity(x, y, z, entity);
                        }
                    }
                }
            })
            .insert(physics_updater);
    }
}
