use serde::{Deserialize, Serialize};
use turbopack_binding::turbo::tasks::{self as turbo_tasks, RcStr, TaskInput};

use crate::{
    id::{EmojiId, RoleId},
    user::User,
};

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Clone, Debug, Serialize, Deserialize, TaskInput, Hash)]
pub struct Emoji {
    /// emoji id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<EmojiId>,
    /// emoji name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<RcStr>,
    /// roles allowed to use this emoji
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<RoleId>>,
    /// user that created this emoji
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
    /// whether this emoji must be wrapped in colons
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_colons: Option<bool>,
    /// whether this emoji is managed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed: Option<bool>,
    /// whether this emoji is animated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animated: Option<bool>,
    /// whether this emoji can be used, may be false due to loss of Server
    /// Boosts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available: Option<bool>,
}
