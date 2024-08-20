use std::fmt::Display;

use serde::{Deserialize, Serialize};
use turbopack_binding::turbo::tasks::{self as turbo_tasks, RcStr, TaskInput, Vc};

pub mod embed;
pub mod message;

use crate::id::{AttachmentId, RoleId, UserId};

#[turbo_tasks::value(shared, serialization = "custom", eq = "manual")]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, TaskInput, Hash)]
pub enum ChannelTypes {
    /// a text channel within a server
    GuildText = 0,
    /// a direct message between users
    DM = 1,
    /// a voice channel within a server
    GuildVoice = 2,
    /// a direct message between multiple users
    GroupDM = 3,
    /// an organizational category that contains up to 50 channels
    GuildCategory = 4,
    /// a channel that users can follow and crosspost into their own server
    /// (formerly news channels)
    GuildAnnouncement = 5,
    /// a temporary sub-channel within a GUILD_ANNOUNCEMENT channel
    AnnouncementThread = 10,
    /// a temporary sub-channel within a GUILD_TEXT or GUILD_FORUM channel
    PublicThread = 11,
    /// a temporary sub-channel within a GUILD_TEXT channel that is only
    /// viewable by those invited and those with the MANAGE_THREADS permission
    PrivateThread = 12,
    /// a voice channel for hosting events with an audience
    GuildStageVoice = 13,
    /// the channel in a hub containing the listed servers
    GuildDirectory = 14,
    /// Channel that can only contain threads
    GuildForum = 15,
    /// Channel that can only contain threads, similar to GUILD_FORUM channels
    GuildMedia = 16,
}

#[turbo_tasks::value]
pub struct ChannelTypesVc(Vc<ChannelTypes>);

// macro to make Attachment only have the id field as required (everything else
// is optional)
macro_rules! extend_struct_mark_rest_optional {
    ($struct_name:ident {
        $(
            // $(#[$field_attr:meta])*
            $field_name:ident: $field_type:ty,
        )*
    }) => {
        #[turbo_tasks::value(shared, serialization = "custom")]
        #[derive(Clone, Debug, Serialize, Deserialize, TaskInput, Hash)]
        pub struct $struct_name {
            /// attachment id
            pub id: AttachmentId,
            $(
                pub $field_name: Option<$field_type>,
            )*
        }
    };
}

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Clone, Debug, Serialize, Deserialize, TaskInput, Hash)]
pub enum Attachment {
    MessageCreateOrEdit(AttachmentForMessageCreateOrEdit),
    Other(AttachmentOther),
}

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Clone, Debug, Serialize, Deserialize, TaskInput, Hash)]
pub struct AttachmentOther {
    /// attachment id
    pub id: AttachmentId,
    /// name of file attached
    pub filename: RcStr,
    /// description for the file (max 1024 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<RcStr>,
    /// the attachment's media type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<RcStr>,
    /// size of file in bytes
    pub size: i32,
    /// source url of file
    pub url: RcStr,
    /// a proxied url of file
    pub proxy_url: RcStr,
    /// height of file (if image)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    /// width of file (if image)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    /// whether this attachment is ephemeral
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ephemeral: Option<bool>,
    /// the duration of the audio file (currently for voice messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_secs: Option<i32>,
    /// base64 encoded bytearray representing a sampled waveform (currently for
    /// voice messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waveform: Option<RcStr>,
    /// attachment flags combined as a bitfield
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<i32>,
}

extend_struct_mark_rest_optional!(AttachmentForMessageCreateOrEdit {
    filename: RcStr,
    description: RcStr,
    content_type: RcStr,
    size: i32,
    url: RcStr,
    proxy_url: RcStr,
    height: i32,
    width: i32,
    ephemeral: bool,
    duration_secs: i32,
    waveform: RcStr,
    flags: i32,
});

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Clone, Debug, Serialize, Deserialize, TaskInput, Hash)]
pub enum AllowedMentionTypes {
    /// Controls role mentions
    Roles,
    /// Controls user mentions
    Users,
    /// Controls @everyone and @here mentions
    Everyone,
}

impl Display for AllowedMentionTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AllowedMentionTypes::Roles => write!(f, "roles"),
            AllowedMentionTypes::Users => write!(f, "users"),
            AllowedMentionTypes::Everyone => write!(f, "everyone"),
        }
    }
}

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Clone, Debug, Serialize, Deserialize, TaskInput, Hash)]
pub struct AllowedMentions {
    /// An array of allowed mention types to parse from the content.
    pub parse: Vec<AllowedMentionTypes>,
    /// Array of role_ids to mention (Max size of 100)
    pub roles: Vec<RoleId>,
    /// Array of user_ids to mention (Max size of 100)
    pub users: Vec<UserId>,
    /// For replies, whether to mention the author of the message being replied
    /// to (default false)
    pub replied_user: bool,
}
