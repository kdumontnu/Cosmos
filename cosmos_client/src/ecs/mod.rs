use bevy::prelude::{
    resource_exists, App, Children, CoreSet, Entity, IntoSystemConfig, Query, ResMut, With,
};
use cosmos_core::ecs::{despawn_needed, NeedsDespawned};

use crate::netty::mapping::NetworkMapping;

/// Recursively removes the networking mappings to all entities that are about to be despawned
pub fn remove_mappings(
    needs_despawned_query: Query<Entity, With<NeedsDespawned>>,
    children_query: Query<&Children>,
    mut network_mapping: ResMut<NetworkMapping>,
) {
    for ent in needs_despawned_query.iter() {
        recursively_remove(ent, &children_query, &mut network_mapping);
    }
}

fn recursively_remove(
    entity: Entity,
    children_query: &Query<&Children>,
    network_mapping: &mut NetworkMapping,
) {
    if let Ok(children) = children_query.get(entity) {
        children
            .iter()
            .copied()
            .for_each(|c| recursively_remove(c, children_query, network_mapping));
    }

    network_mapping.remove_mapping_from_client_entity(&entity);
}

pub(super) fn register(app: &mut App) {
    app.add_system(
        remove_mappings
            .in_base_set(CoreSet::First)
            .before(despawn_needed)
            .run_if(resource_exists::<NetworkMapping>()),
    );
}
