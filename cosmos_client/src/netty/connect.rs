use std::{
    net::UdpSocket,
    time::{SystemTime, UNIX_EPOCH},
};

use bevy::prelude::*;
use bevy_renet::renet::{ClientAuthentication, RenetClient};
use cosmos_core::{
    entities::player::Player,
    netty::{client_connection_config, PROTOCOL_ID},
};

use crate::{
    netty::{
        lobby::{ClientLobby, MostRecentTick},
        mapping::NetworkMapping,
    },
    state::game_state::GameState,
};

use super::flags::LocalPlayer;

fn new_renet_client(host: &str) -> RenetClient {
    let port: u16 = 1337;

    let server_addr = format!("{host}:{port}").parse().unwrap();
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

    socket
        .set_nonblocking(true)
        .expect("Unable to make UDP non-blocking!");

    let connection_config = client_connection_config();
    let cur_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let client_id = cur_time.as_millis() as u64;

    let name = "CoolPlayer";

    let mut token = [0; 256];

    let serialized_name = bincode::serialize(name).unwrap();
    for (i, byte) in serialized_name.iter().enumerate() {
        token[i] = *byte;
    }

    let auth = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: Some(token),
    };

    println!("Connecting to {server_addr}");

    RenetClient::new(cur_time, socket, connection_config, auth).unwrap()
}

#[derive(Resource)]
pub struct ConnectionConfig {
    pub host_name: String,
}

pub fn establish_connection(mut commands: Commands, connection_config: Res<ConnectionConfig>) {
    println!("Establishing connection w/ server...");
    commands.insert_resource(ClientLobby::default());
    commands.insert_resource(MostRecentTick(None));
    commands.insert_resource(new_renet_client(connection_config.host_name.as_str()));
    commands.insert_resource(NetworkMapping::default());
}

pub fn wait_for_connection(mut state: ResMut<State<GameState>>, client: Res<RenetClient>) {
    if client.is_connected() {
        println!("Loading server data...");
        state.set(GameState::LoadingWorld).unwrap();
    }
}

pub fn wait_for_done_loading(
    mut state: ResMut<State<GameState>>,
    query: Query<&Player, With<LocalPlayer>>,
) {
    if query.get_single().is_ok() {
        println!("Got local player, starting game!");
        state
            .set(GameState::Playing)
            .expect("Unable to change state into playing");
    }
}
