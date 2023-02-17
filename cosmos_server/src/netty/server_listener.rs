use bevy::prelude::*;
use bevy_renet::renet::RenetServer;
use cosmos_core::physics::location::{sync_translations, Location};
use cosmos_core::structure::systems::{SystemActive, Systems};
use cosmos_core::{
    entities::player::Player,
    events::structure::change_pilot_event::ChangePilotEvent,
    netty::{
        client_reliable_messages::ClientReliableMessages,
        client_unreliable_messages::ClientUnreliableMessages,
        server_reliable_messages::ServerReliableMessages, NettyChannel,
    },
    structure::{
        ship::pilot::Pilot,
        {structure_block::StructureBlock, Structure},
    },
};

use crate::entities::player::PlayerLooking;
use crate::events::{
    blocks::block_events::{BlockBreakEvent, BlockInteractEvent, BlockPlaceEvent},
    create_ship_event::CreateShipEvent,
    structure::ship::ShipSetMovementEvent,
};

use super::network_helpers::ServerLobby;

fn server_listen_messages(
    mut commands: Commands,
    mut server: ResMut<RenetServer>,
    lobby: ResMut<ServerLobby>,
    structure_query: Query<&Structure>,
    mut systems_query: Query<&mut Systems>,
    mut break_block_event: EventWriter<BlockBreakEvent>,
    mut block_interact_event: EventWriter<BlockInteractEvent>,
    mut place_block_event: EventWriter<BlockPlaceEvent>,
    mut create_ship_event_writer: EventWriter<CreateShipEvent>,

    mut ship_movement_event_writer: EventWriter<ShipSetMovementEvent>,
    mut pilot_change_event_writer: EventWriter<ChangePilotEvent>,
    pilot_query: Query<&Pilot>,
    mut change_player_query: Query<(&mut Location, &mut PlayerLooking), With<Player>>,
) {
    for client_id in server.clients_id().into_iter() {
        while let Some(message) = server.receive_message(client_id, NettyChannel::Unreliable.id()) {
            if let Some(player_entity) = lobby.players.get(&client_id) {
                let command: ClientUnreliableMessages = bincode::deserialize(&message).unwrap();

                match command {
                    ClientUnreliableMessages::PlayerBody { body, looking } => {
                        if let Ok((mut location, mut currently_looking)) =
                            change_player_query.get_mut(*player_entity)
                        {
                            location.set_from(&body.location);
                            currently_looking.rotation = looking;
                        }
                    }
                    ClientUnreliableMessages::SetMovement { movement } => {
                        if let Ok(pilot) = pilot_query.get(*player_entity) {
                            let ship = pilot.entity;

                            ship_movement_event_writer
                                .send(ShipSetMovementEvent { movement, ship });
                        }
                    }
                    ClientUnreliableMessages::ShipStatus { use_system } => {
                        if let Ok(pilot) = pilot_query.get(*player_entity) {
                            if use_system {
                                commands.entity(pilot.entity).insert(SystemActive);
                            } else {
                                commands.entity(pilot.entity).remove::<SystemActive>();
                            }
                        }
                    }
                    ClientUnreliableMessages::ShipActiveSystem { active_system } => {
                        if let Ok(pilot) = pilot_query.get(*player_entity) {
                            if let Ok(mut systems) = systems_query.get_mut(pilot.entity) {
                                systems.set_active_system(active_system, &mut commands);
                            }
                        }
                    }
                }
            }
        }

        while let Some(message) = server.receive_message(client_id, NettyChannel::Reliable.id()) {
            let command: ClientReliableMessages = bincode::deserialize(&message).unwrap();

            match command {
                ClientReliableMessages::PlayerDisconnect => {}
                ClientReliableMessages::SendChunk { server_entity } => {
                    if let Ok(structure) = structure_query.get(server_entity) {
                        for chunk in structure.chunks() {
                            server.send_message(
                                client_id,
                                NettyChannel::Reliable.id(),
                                bincode::serialize(&ServerReliableMessages::ChunkData {
                                    structure_entity: server_entity,
                                    serialized_chunk: bincode::serialize(chunk).unwrap(),
                                })
                                .unwrap(),
                            );
                        }
                    } else {
                        println!("!!! Server received invalid entity from client {client_id}");
                    }
                }
                ClientReliableMessages::BreakBlock {
                    structure_entity,
                    x,
                    y,
                    z,
                } => {
                    if let Some(player_entity) = lobby.players.get(&client_id) {
                        break_block_event.send(BlockBreakEvent {
                            structure_entity,
                            breaker: *player_entity,
                            x: x as usize,
                            y: y as usize,
                            z: z as usize,
                        });
                    }
                }
                ClientReliableMessages::PlaceBlock {
                    structure_entity,
                    x,
                    y,
                    z,
                    block_id,
                    inventory_slot,
                } => {
                    if let Some(player_entity) = lobby.players.get(&client_id) {
                        place_block_event.send(BlockPlaceEvent {
                            structure_entity,
                            x: x as usize,
                            y: y as usize,
                            z: z as usize,
                            block_id,
                            inventory_slot: inventory_slot as usize,
                            placer: *player_entity,
                        });
                    }
                }
                ClientReliableMessages::InteractWithBlock {
                    structure_entity,
                    x,
                    y,
                    z,
                } => {
                    block_interact_event.send(BlockInteractEvent {
                        structure_entity,
                        structure_block: StructureBlock::new(x as usize, y as usize, z as usize),
                        interactor: *lobby.players.get(&client_id).unwrap(),
                    });
                }
                ClientReliableMessages::CreateShip { name: _name } => {
                    if let Some(client) = lobby.players.get(&client_id) {
                        let (location, looking) = change_player_query.get(*client).unwrap();

                        let ship_location =
                            *location + looking.rotation.mul_vec3(Vec3::new(0.0, 0.0, -4.0));

                        create_ship_event_writer.send(CreateShipEvent {
                            ship_location,
                            rotation: looking.rotation,
                        });
                    }
                }
                ClientReliableMessages::PilotQuery { ship_entity } => {
                    let pilot = match pilot_query.get(ship_entity) {
                        Ok(pilot) => Some(pilot.entity),
                        _ => None,
                    };

                    server.send_message(
                        client_id,
                        NettyChannel::Reliable.id(),
                        bincode::serialize(&ServerReliableMessages::PilotChange {
                            structure_entity: ship_entity,
                            pilot_entity: pilot,
                        })
                        .unwrap(),
                    );
                }
                ClientReliableMessages::StopPiloting => {
                    if let Some(player_entity) = lobby.players.get(&client_id) {
                        if let Ok(piloting) = pilot_query.get(*player_entity) {
                            pilot_change_event_writer.send(ChangePilotEvent {
                                structure_entity: piloting.entity,
                                pilot_entity: None,
                            });
                        }
                    }
                }
            }
        }
    }
}

pub fn register(app: &mut App) {
    app.add_system(server_listen_messages)
        // If it's not after this system, some noticable jitter can happen
        .add_system(sync_translations.after(server_listen_messages));
}
