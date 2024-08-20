use serde::{Deserialize, Serialize};
use turbopack_binding::turbo::tasks as turbo_tasks;

use crate::id::GuildId;

#[turbo_tasks::value(shared, serialization = "custom", eq = "manual")]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UnavailableGuild {
    pub id: GuildId,
    pub unavailable: bool,
}

#[turbo_tasks::value(shared, serialization = "custom", eq = "manual")]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Guild {}
