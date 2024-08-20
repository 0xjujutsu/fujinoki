use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RESTGateway {
    wss: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RESTGatewayBot {
    /// WSS URL that can be used for connecting to the Gateway
    url: String,
    /// Recommended number of shards to use when connecting
    shards: u32,
    /// Information on the current session start limit
    session_start_limit: SessionStartLimit,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SessionStartLimit {
    /// Total number of session starts the current user is allowed
    total: u32,
    /// Remaining number of session starts the current user is allowed
    remaining: u32,
    /// Number of milliseconds after which the limit resets
    reset_after: u32,
    /// Number of identify requests allowed per 5 seconds
    max_concurrency: u32,
}
