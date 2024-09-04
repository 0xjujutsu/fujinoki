// TODO Do rest of these

use serde::{Deserialize, Serialize};
use turbopack_binding::turbo::tasks as turbo_tasks;

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifyPayloadData {
    /// Authentication token
    pub token: String,

    /// Connection properties
    pub properties: ConnectionProperties,

    /// Whether this connection supports compression of packets
    #[serde(default)]
    pub compress: Option<bool>,

    /// Value between 50 and 250, total number of members where the gateway will stop sending offline members in the guild member list
    #[serde(default)]
    pub large_threshold: Option<u8>,

    /// Used for Guild Sharding
    #[serde(default)]
    pub shard: Option<[u32; 2]>,

    /// Presence structure for initial presence information
    // #[serde(default)]
    // pub presence: Option<UpdatePresence>,

    /// Gateway Intents you wish to receive
    pub intents: u32,
}

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionProperties {
    /// Operating system
    pub os: String,

    /// Library name
    pub browser: String,

    /// Library name (same as browser)
    pub device: String,
}

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct Activity {
    // TODO
}

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePresencePayloadData {
    /// Unix time (in milliseconds) of when the client went idle, or null if the client is not idle
    #[serde(default)]
    pub since: Option<u64>,

    /// User's activities
    pub activities: Vec<Activity>,

    /// User's new status
    pub status: PresenceStatus,

    /// Whether or not the client is afk
    pub afk: bool,
}

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PresenceStatus {
    #[serde(rename = "online")]
    Online,
    #[serde(rename = "dnd")]
    DoNotDisturb,
    #[serde(rename = "idle")]
    Idle,
    #[serde(rename = "invisible")]
    Invisible,
    #[serde(rename = "offline")]
    Offline,
}

impl Into<String> for PresenceStatus {
    fn into(self) -> String {
        match self {
            PresenceStatus::Online => "online".to_string(),
            PresenceStatus::DoNotDisturb => "dnd".to_string(),
            PresenceStatus::Idle => "idle".to_string(),
            PresenceStatus::Invisible => "invisible".to_string(),
            PresenceStatus::Offline => "offline".to_string(),
        }
    }
}
