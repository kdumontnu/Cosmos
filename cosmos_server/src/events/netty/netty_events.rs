//! Handles client connecting and disconnecting

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_renet::renet::transport::NetcodeServerTransport;
use bevy_renet::renet::{RenetServer, ServerEvent};
use cosmos_core::ecs::NeedsDespawned;
use cosmos_core::entities::player::render_distance::RenderDistance;
use cosmos_core::inventory::Inventory;
use cosmos_core::item::Item;
use cosmos_core::netty::netty_rigidbody::NettyRigidBodyLocation;
use cosmos_core::netty::server_reliable_messages::ServerReliableMessages;
use cosmos_core::netty::{cosmos_encoder, NettyChannelServer};
use cosmos_core::persistence::LoadingDistance;
use cosmos_core::physics::location::{Location, Sector};
use cosmos_core::physics::player_world::WorldWithin;
use cosmos_core::registry::Registry;
use cosmos_core::structure::chunk::CHUNK_DIMENSIONSF;
use cosmos_core::{entities::player::Player, netty::netty_rigidbody::NettyRigidBody};
use renet_visualizer::RenetServerVisualizer;

use crate::entities::player::PlayerLooking;
use crate::netty::network_helpers::{ClientTicks, ServerLobby};

fn generate_player_inventory(items: &Registry<Item>) -> Inventory {
    let mut inventory = Inventory::new(9);

    inventory.insert_at(0, items.from_id("cosmos:ice").expect("Ice item to exist"), 999);

    inventory.insert_at(1, items.from_id("cosmos:water").expect("Water item to exist"), 999);

    inventory.insert_at(2, items.from_id("cosmos:glass").expect("Glass item to exist"), 64);

    inventory.insert_at(3, items.from_id("cosmos:thruster").expect("Thruster item to exist"), 64);

    inventory.insert_at(4, items.from_id("cosmos:laser_cannon").expect("Laser cannon item to exist"), 64);

    inventory.insert_at(5, items.from_id("cosmos:reactor").expect("Reactor cannon item to exist"), 64);

    inventory.insert_at(6, items.from_id("cosmos:energy_cell").expect("Energy cell item to exist"), 64);

    inventory.insert_at(7, items.from_id("cosmos:ship_hull").expect("Ship hull item to exist"), 999);

    inventory.insert_at(8, items.from_id("cosmos:light").expect("Light item to exist"), 64);

    inventory
}

use crate::netty::sync::entities::RequestedEntityEvent;
use crate::physics::assign_player_world;
use crate::state::GameState;

fn handle_events_system(
    mut commands: Commands,
    mut server: ResMut<RenetServer>,
    transport: Res<NetcodeServerTransport>,
    mut server_events: EventReader<ServerEvent>,
    mut lobby: ResMut<ServerLobby>,
    mut client_ticks: ResMut<ClientTicks>,
    players: Query<(Entity, &Player, &Transform, &Location, &Velocity, &Inventory, &RenderDistance)>,
    player_worlds: Query<(&Location, &WorldWithin, &PhysicsWorld), (With<Player>, Without<Parent>)>,
    items: Res<Registry<Item>>,
    mut visualizer: ResMut<RenetServerVisualizer<200>>,
    mut rapier_context: ResMut<RapierContext>,
    mut requested_entity: EventWriter<RequestedEntityEvent>,
) {
    for event in server_events.iter() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                let client_id = *client_id;
                println!("Client {client_id} connected");
                visualizer.add_client(client_id);

                for (entity, player, transform, location, velocity, inventory, render_distance) in players.iter() {
                    let body = NettyRigidBody::new(velocity, transform.rotation, NettyRigidBodyLocation::Absolute(*location));

                    let msg = cosmos_encoder::serialize(&ServerReliableMessages::PlayerCreate {
                        entity,
                        id: player.id(),
                        body,
                        name: player.name().clone(),
                        inventory_serialized: cosmos_encoder::serialize(inventory),
                        render_distance: Some(*render_distance),
                    });

                    server.send_message(client_id, NettyChannelServer::Reliable, msg);
                    requested_entity.send(RequestedEntityEvent { client_id, entity });
                }

                let Some(user_data) = transport.user_data(client_id) else {
                    println!("Unable to get user data!");
                    continue;
                };
                let Ok(name) = bincode::deserialize::<String>(user_data.as_slice()) else {
                    println!("Unable to deserialize name!");
                    continue;
                };

                let player = Player::new(name.clone(), client_id);
                let starting_pos = Vec3::new(0.0, CHUNK_DIMENSIONSF * 64.0 / 2.0, 0.0);
                let location = Location::new(starting_pos, Sector::new(25, 25, 25));
                let velocity = Velocity::default();
                let inventory = generate_player_inventory(&items);

                let netty_body = NettyRigidBody::new(&velocity, Quat::IDENTITY, NettyRigidBodyLocation::Absolute(location));

                let inventory_serialized = cosmos_encoder::serialize(&inventory);

                let player_commands = commands.spawn((
                    location,
                    LockedAxes::ROTATION_LOCKED,
                    RigidBody::Dynamic,
                    velocity,
                    Collider::capsule_y(0.5, 0.25),
                    player,
                    ReadMassProperties::default(),
                    inventory,
                    Ccd::enabled(),
                    PlayerLooking { rotation: Quat::IDENTITY },
                    LoadingDistance::new(2, 9999),
                    ActiveEvents::COLLISION_EVENTS,
                ));

                let entity = player_commands.id();

                lobby.add_player(client_id, entity);

                assign_player_world(&player_worlds, entity, &location, &mut commands, &mut rapier_context);

                let msg = cosmos_encoder::serialize(&ServerReliableMessages::PlayerCreate {
                    entity,
                    id: client_id,
                    name,
                    body: netty_body,
                    inventory_serialized,
                    render_distance: None,
                });

                server.send_message(
                    client_id,
                    NettyChannelServer::Reliable,
                    cosmos_encoder::serialize(&ServerReliableMessages::MOTD {
                        motd: "Welcome to the server!".into(),
                    }),
                );

                server.broadcast_message(NettyChannelServer::Reliable, msg);
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                println!("Client {client_id} disconnected: {reason}");
                visualizer.remove_client(*client_id);
                client_ticks.ticks.remove(client_id);

                if let Some(player_entity) = lobby.remove_player(*client_id) {
                    commands.entity(player_entity).insert(NeedsDespawned);
                }

                let message = cosmos_encoder::serialize(&ServerReliableMessages::PlayerRemove { id: *client_id });

                server.broadcast_message(NettyChannelServer::Reliable, message);
            }
        }
    }
}

pub(super) fn register(app: &mut App) {
    app.add_systems(Update, handle_events_system.run_if(in_state(GameState::Playing)));
}
