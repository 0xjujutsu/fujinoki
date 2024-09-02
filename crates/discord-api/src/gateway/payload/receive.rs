// TODO Do rest of these

use std::time::Duration;

use serde::{Deserialize, Serialize};
use turbopack_binding::turbo::tasks as turbo_tasks;

use crate::{application::PartialApplication, guild::UnavailableGuild, user::User};

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloPayloadData {
    pub heartbeat_interval: Duration,
}

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadyPayloadData {
    /// API version
    v: u8,
    /// Information about the user including email
    user: User,
    /// Guilds the user is in
    guilds: Vec<UnavailableGuild>,
    /// Used for resuming connections
    session_id: String,
    /// Gateway URL for resuming connections
    resume_gateway_url: String,
    /// Shard information associated with this session, if sent when identifying
    shard: (i8, i8),
    /// Contains id and flags
    application: PartialApplication,
}
