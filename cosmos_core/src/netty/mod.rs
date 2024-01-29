//! Contains all the information required for network requests

pub mod client_reliable_messages;
pub mod client_unreliable_messages;
pub mod cosmos_encoder;
pub mod netty_rigidbody;
pub mod server_laser_cannon_system_messages;
pub mod server_registry;
pub mod server_reliable_messages;
pub mod server_replication;
pub mod server_unreliable_messages;
pub mod system_sets;
pub mod world_tick;

use bevy::prelude::{App, Component};
use bevy_renet::renet::{ChannelConfig, ConnectionConfig, SendType};
use local_ip_address::local_ip;
use std::time::Duration;

/// Used to tell the server to not send this entity to the player
///
/// Useful for entities that are automatically generated by other entities (like chunks)
#[derive(Component)]
pub struct NoSendEntity;

/// Network channels that the server sends to clients
pub enum NettyChannelServer {
    /// These are reliably sent, so they are guarenteed to reach their destination.
    /// Used for sending `ServerReliableMessages`
    Reliable,
    /// These are unreliably sent, and may never reach their destination or become corrupted.
    /// Used for sending `ServerUnreliableMessages`
    Unreliable,
    /// Used for `ServerLaserCannonSystemMessages`
    LaserCannonSystem,
    /// Used for asteroids
    Asteroid,
    /// Sending LOD information to the client
    DeltaLod,
    /// Used for inventories
    Inventory,
    /// In future will be used for general component syncing
    SystemReplication,
    /// Syncing of registry data
    Registry,
}

/// Network channels that clients send to the server
pub enum NettyChannelClient {
    /// These are reliably sent, so they are guarenteed to reach their destination.
    /// Used for sending `ClientReliableMessages`
    Reliable,
    /// These are unreliably sent, and may never reach their destination or become corrupted.
    /// Used for sending `ClientUnreliableMessages`
    Unreliable,
    /// used for inventories
    Inventory,
}

impl From<NettyChannelClient> for u8 {
    fn from(channel_id: NettyChannelClient) -> Self {
        match channel_id {
            NettyChannelClient::Reliable => 0,
            NettyChannelClient::Unreliable => 1,
            NettyChannelClient::Inventory => 2,
        }
    }
}

const KB: usize = 1024;
const MB: usize = KB * KB;

impl NettyChannelClient {
    /// Assembles & returns the configuration for all the client channels
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![
            ChannelConfig {
                channel_id: Self::Reliable.into(),
                max_memory_usage_bytes: 5 * MB,
                send_type: SendType::ReliableOrdered {
                    resend_time: Duration::from_millis(200),
                },
            },
            ChannelConfig {
                channel_id: Self::Unreliable.into(),
                max_memory_usage_bytes: 5 * MB,
                send_type: SendType::Unreliable,
            },
            ChannelConfig {
                channel_id: Self::Inventory.into(),
                max_memory_usage_bytes: 5 * MB,
                send_type: SendType::ReliableOrdered {
                    resend_time: Duration::from_millis(200),
                },
            },
        ]
    }
}

impl From<NettyChannelServer> for u8 {
    fn from(channel_id: NettyChannelServer) -> Self {
        match channel_id {
            NettyChannelServer::Reliable => 0,
            NettyChannelServer::Unreliable => 1,
            NettyChannelServer::LaserCannonSystem => 2,
            NettyChannelServer::Asteroid => 3,
            NettyChannelServer::DeltaLod => 4,
            NettyChannelServer::Inventory => 5,
            NettyChannelServer::SystemReplication => 6,
            NettyChannelServer::Registry => 7,
        }
    }
}

impl NettyChannelServer {
    /// Assembles & returns the config for all the server channels
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![
            ChannelConfig {
                channel_id: Self::Reliable.into(),
                max_memory_usage_bytes: 5 * MB,
                send_type: SendType::ReliableOrdered {
                    resend_time: Duration::from_millis(200),
                },
            },
            ChannelConfig {
                channel_id: Self::Unreliable.into(),
                max_memory_usage_bytes: 5 * MB,
                send_type: SendType::Unreliable,
            },
            ChannelConfig {
                channel_id: Self::LaserCannonSystem.into(),
                max_memory_usage_bytes: 5 * MB,
                send_type: SendType::Unreliable,
            },
            ChannelConfig {
                channel_id: Self::Asteroid.into(),
                max_memory_usage_bytes: 5 * MB,
                send_type: SendType::ReliableOrdered {
                    resend_time: Duration::from_millis(200),
                },
            },
            ChannelConfig {
                channel_id: Self::Inventory.into(),
                max_memory_usage_bytes: 5 * MB,
                send_type: SendType::ReliableOrdered {
                    resend_time: Duration::from_millis(200),
                },
            },
            ChannelConfig {
                channel_id: Self::DeltaLod.into(),
                max_memory_usage_bytes: 5 * MB,
                send_type: SendType::ReliableOrdered {
                    resend_time: Duration::from_millis(200),
                },
            },
            ChannelConfig {
                channel_id: Self::SystemReplication.into(),
                max_memory_usage_bytes: 5 * MB,
                send_type: SendType::ReliableOrdered {
                    resend_time: Duration::from_millis(200),
                },
            },
            ChannelConfig {
                channel_id: Self::Registry.into(),
                max_memory_usage_bytes: 5 * MB,
                send_type: SendType::ReliableOrdered {
                    resend_time: Duration::from_millis(200),
                },
            },
        ]
    }
}

/// In the future, this should be based off the game version.
///
/// Must have the same protocol to connect to something
pub const PROTOCOL_ID: u64 = 7;

/// Assembles the configuration for a renet connection
pub fn connection_config() -> ConnectionConfig {
    ConnectionConfig {
        available_bytes_per_tick: MB as u64,
        client_channels_config: NettyChannelClient::channels_config(),
        server_channels_config: NettyChannelServer::channels_config(),
    }
}

/// Gets the local ip address, or returns `127.0.0.1` if it fails to find it.
pub fn get_local_ipaddress() -> String {
    local_ip().map(|x| x.to_string()).unwrap_or("127.0.0.1".to_owned())
}

pub(super) fn register(app: &mut App) {
    world_tick::register(app);
    system_sets::register(app);
}
