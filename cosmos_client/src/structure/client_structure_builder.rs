use bevy::ecs::system::EntityCommands;
use cosmos_core::{
    physics::location::Location,
    structure::{
        loading::ChunksNeedLoaded, structure_builder::StructureBuilder,
        structure_builder::TStructureBuilder,
    },
};

use crate::rendering::structure_renderer::StructureRenderer;

use super::chunk_retreiver::NeedsPopulated;

#[derive(Default)]
pub struct ClientStructureBuilder {
    structure_builder: StructureBuilder,
}

impl TStructureBuilder for ClientStructureBuilder {
    fn insert_structure(
        &self,
        entity: &mut EntityCommands,
        location: Location,
        velocity: bevy_rapier3d::prelude::Velocity,
        structure: &mut cosmos_core::structure::Structure,
    ) {
        self.structure_builder
            .insert_structure(entity, location, velocity, structure);

        let renderer = StructureRenderer::new(structure);

        entity.insert((
            renderer,
            NeedsPopulated,
            ChunksNeedLoaded { amount_needed: 1 }, // not the best solution, but it works
        ));
    }
}
