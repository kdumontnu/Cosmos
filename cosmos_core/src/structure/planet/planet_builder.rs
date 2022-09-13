use bevy::{ecs::system::EntityCommands, prelude::Transform};
use bevy_rapier3d::prelude::{RigidBody, Velocity};

use crate::structure::{structure::Structure, structure_builder::TStructureBuilder};

use super::planet::Planet;

pub trait TPlanetBuilder {
    fn insert_planet(
        &self,
        entity: &mut EntityCommands,
        transform: Transform,
        structure: &mut Structure,
    );
}

pub struct PlanetBuilder<T: TStructureBuilder> {
    structure_builder: T,
}

impl<T: TStructureBuilder> PlanetBuilder<T> {
    pub fn new(structure_builder: T) -> Self {
        Self { structure_builder }
    }
}

impl<T: TStructureBuilder> TPlanetBuilder for PlanetBuilder<T> {
    fn insert_planet(
        &self,
        entity: &mut EntityCommands,
        transform: Transform,
        structure: &mut Structure,
    ) {
        self.structure_builder
            .insert_structure(entity, transform, Velocity::default(), structure);

        entity.insert(Planet).insert(RigidBody::Fixed);
    }
}
