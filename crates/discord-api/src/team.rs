use serde::{Deserialize, Serialize};
use turbopack_binding::turbo::tasks as turbo_tasks;

use crate::{
    id::{GenericId, UserId},
    user::User,
};

#[turbo_tasks::value(shared, serialization = "custom", eq = "manual")]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Team {
    /// Hash of the image of the team's icon
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
    /// Unique ID of the team
    id: GenericId,
    /// Members of the team
    members: Vec<TeamMember>,
    /// Name of the team
    name: String,
    /// User ID of the current team owner
    owner_user_id: UserId,
}

#[turbo_tasks::value(shared, serialization = "custom", eq = "manual")]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TeamMember {
    /// User's membership state on the team
    membership_state: i32,
    /// ID of the parent team of which they are a member
    team_id: GenericId,
    /// Avatar, discriminator, ID, and username of the user
    user: User,
    /// Role of the team member
    role: String,
}
