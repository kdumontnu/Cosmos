//! Player interactions with structures

use bevy::{
    prelude::{in_state, Added, App, Commands, Component, Entity, IntoSystemConfigs, Query, RemovedComponents, ResMut, Update, With},
    reflect::Reflect,
};
use bevy_renet::renet::RenetClient;
use cosmos_core::{
    netty::{client::LocalPlayer, client_unreliable_messages::ClientUnreliableMessages, cosmos_encoder, NettyChannelClient},
    structure::{ship::pilot::Pilot, systems::ShipActiveSystem},
};

use crate::{
    input::inputs::{CosmosInputs, InputChecker, InputHandler},
    state::game_state::GameState,
    ui::components::show_cursor::no_open_menus,
};

#[derive(Component, Default, Reflect)]
/// Contains the structure system's information currently hovered by the player
pub struct HoveredSystem {
    /// The index of the system, relative to the `active_systems` iterator
    pub hovered_system_index: usize,
    /// If the hovered system is active
    pub active: bool,
}

fn check_system_in_use(
    mut query: Query<&mut HoveredSystem, (With<Pilot>, With<LocalPlayer>)>,
    input_handler: InputChecker,
    mut client: ResMut<RenetClient>,
) {
    let Ok(mut hovered_system) = query.get_single_mut() else {
        return;
    };

    hovered_system.active = input_handler.check_pressed(CosmosInputs::UseSelectedSystem);

    let active_system = if hovered_system.active {
        ShipActiveSystem::Active(hovered_system.hovered_system_index as u32)
    } else {
        ShipActiveSystem::Hovered(hovered_system.hovered_system_index as u32)
    };

    client.send_message(
        NettyChannelClient::Unreliable,
        cosmos_encoder::serialize(&ClientUnreliableMessages::ShipActiveSystem(active_system)),
    );
}

fn check_became_pilot(mut commands: Commands, query: Query<Entity, (Added<Pilot>, With<LocalPlayer>)>) {
    for ent in query.iter() {
        commands.entity(ent).insert(HoveredSystem::default());
    }
}

fn check_removed_pilot(mut commands: Commands, mut removed: RemovedComponents<Pilot>) {
    for ent in removed.read() {
        if let Some(mut ecmds) = commands.get_entity(ent) {
            ecmds.remove::<HoveredSystem>();
        }
    }
}

pub(super) fn register(app: &mut App) {
    app.add_systems(
        Update,
        (check_system_in_use.run_if(no_open_menus), check_became_pilot, check_removed_pilot).run_if(in_state(GameState::Playing)),
    )
    .register_type::<HoveredSystem>();
}
