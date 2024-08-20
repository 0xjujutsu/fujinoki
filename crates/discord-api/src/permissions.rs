use bitflags::bitflags;
use serde::{Deserialize, Deserializer, Serialize};
use turbopack_binding::turbo::tasks as turbo_tasks;

#[derive(Debug, Default)]
pub struct Permissions(pub u64);

impl Permissions {
    pub fn from_str(s: &str) -> Option<Self> {
        let mut perms = Permissions(0);
        for perm in s.split_whitespace() {
            match perm {
                "CREATE_INSTANT_INVITE" => perms.0 |= 1 << 0,
                "KICK_MEMBERS" => perms.0 |= 1 << 1,
                "BAN_MEMBERS" => perms.0 |= 1 << 2,
                "ADMINISTRATOR" => perms.0 |= 1 << 3,
                "MANAGE_CHANNELS" => perms.0 |= 1 << 4,
                "MANAGE_GUILD" => perms.0 |= 1 << 5,
                "ADD_REACTIONS" => perms.0 |= 1 << 6,
                "VIEW_AUDIT_LOG" => perms.0 |= 1 << 7,
                "PRIORITY_SPEAKER" => perms.0 |= 1 << 8,
                "STREAM" => perms.0 |= 1 << 9,
                "VIEW_CHANNEL" => perms.0 |= 1 << 10,
                "SEND_MESSAGES" => perms.0 |= 1 << 11,
                "SEND_TTS_MESSAGES" => perms.0 |= 1 << 12,
                "MANAGE_MESSAGES" => perms.0 |= 1 << 13,
                "EMBED_LINKS" => perms.0 |= 1 << 14,
                "ATTACH_FILES" => perms.0 |= 1 << 15,
                "READ_MESSAGE_HISTORY" => perms.0 |= 1 << 16,
                "MENTION_EVERYONE" => perms.0 |= 1 << 17,
                "USE_EXTERNAL_EMOJIS" => perms.0 |= 1 << 18,
                "VIEW_GUILD_INSIGHTS" => perms.0 |= 1 << 19,
                "CONNECT" => perms.0 |= 1 << 20,
                "SPEAK" => perms.0 |= 1 << 21,
                "MUTE_MEMBERS" => perms.0 |= 1 << 22,
                "DEAFEN_MEMBERS" => perms.0 |= 1 << 23,
                "MOVE_MEMBERS" => perms.0 |= 1 << 24,
                "USE_VAD" => perms.0 |= 1 << 25,
                "CHANGE_NICKNAME" => perms.0 |= 1 << 26,
                "MANAGE_NICKNAMES" => perms.0 |= 1 << 27,
                "MANAGE_ROLES" => perms.0 |= 1 << 28,
                "MANAGE_WEBHOOKS" => perms.0 |= 1 << 29,
                "MANAGE_GUILD_EXPRESSIONS" => perms.0 |= 1 << 30,
                "USE_APPLICATION_COMMANDS" => perms.0 |= 1 << 31,
                "REQUEST_TO_SPEAK" => perms.0 |= 1 << 32,
                "MANAGE_EVENTS" => perms.0 |= 1 << 33,
                "MANAGE_THREADS" => perms.0 |= 1 << 34,
                "CREATE_PUBLIC_THREADS" => perms.0 |= 1 << 35,
                "CREATE_PRIVATE_THREADS" => perms.0 |= 1 << 36,
                "USE_EXTERNAL_STICKERS" => perms.0 |= 1 << 37,
                "SEND_MESSAGES_IN_THREADS" => perms.0 |= 1 << 38,
                "USE_EMBEDDED_ACTIVITIES" => perms.0 |= 1 << 39,
                "MODERATE_MEMBERS" => perms.0 |= 1 << 40,
                "VIEW_CREATOR_MONETIZATION_ANALYTICS" => perms.0 |= 1 << 41,
                "USE_SOUNDBOARD" => perms.0 |= 1 << 42,
                "CREATE_GUILD_EXPRESSIONS" => perms.0 |= 1 << 43,
                "CREATE_EVENTS" => perms.0 |= 1 << 44,
                "USE_EXTERNAL_SOUNDS" => perms.0 |= 1 << 45,
                "SEND_VOICE_MESSAGES" => perms.0 |= 1 << 46,
                _ => return None,
            }
        }
        Some(perms)
    }
}

bitflags! {
    #[turbo_tasks::value(shared, serialization = "custom", eq = "manual")]
    #[derive(Serialize, Default)]
    pub struct PermissionFlags: u64 {
        /// Allows creation of instant invites (T, V, S)
        const CREATE_INSTANT_INVITE = 1 << 0;
        /// Allows kicking members
        const KICK_MEMBERS = 1 << 1;
        /// Allows banning members
        const BAN_MEMBERS = 1 << 2;
        /// Allows all permissions and bypasses channel permission overwrites
        const ADMINISTRATOR = 1 << 3;
        /// Allows management and editing of channels (T, V, S)
        const MANAGE_CHANNELS = 1 << 4;
        /// Allows management and editing of the guild
        const MANAGE_GUILD = 1 << 5;
        /// Allows for the addition of reactions to messages (T, V, S)
        const ADD_REACTIONS = 1 << 6;
        /// Allows for viewing of audit logs
        const VIEW_AUDIT_LOG = 1 << 7;
        /// Allows for using priority speaker in a voice channel (V)
        const PRIORITY_SPEAKER = 1 << 8;
        /// Allows the user to go live (V, S)
        const STREAM = 1 << 9;
        /// Allows guild members to view a channel, which includes reading messages in text channels and joining voice channels (T, V, S)
        const VIEW_CHANNEL = 1 << 10;
        /// Allows for sending messages in a channel and creating threads in a forum (does not allow sending messages in threads) (T, V, S)
        const SEND_MESSAGES = 1 << 11;
        /// Allows for sending of /tts messages (T, V, S)
        const SEND_TTS_MESSAGES = 1 << 12;
        /// Allows for deletion of other users messages (T, V, S)
        const MANAGE_MESSAGES = 1 << 13;
        /// Links sent by users with this permission will be auto-embedded (T, V, S)
        const EMBED_LINKS = 1 << 14;
        /// Allows for uploading images and files (T, V, S)
        const ATTACH_FILES = 1 << 15;
        /// Allows for reading of message history (T, V, S)
        const READ_MESSAGE_HISTORY = 1 << 16;
        /// Allows for using the @everyone tag to notify all users in a channel, and the @here tag to notify all online users in a channel (T, V, S)
        const MENTION_EVERYONE = 1 << 17;
        /// Allows the usage of custom emojis from other servers (T, V, S)
        const USE_EXTERNAL_EMOJIS = 1 << 18;
        /// Allows for viewing guild insights
        const VIEW_GUILD_INSIGHTS = 1 << 19;
        /// Allows for joining of a voice channel (V, S)
        const CONNECT = 1 << 20;
        /// Allows for speaking in a voice channel (V)
        const SPEAK = 1 << 21;
        /// Allows for muting members in a voice channel (V, S)
        const MUTE_MEMBERS = 1 << 22;
        /// Allows for deafening of members in a voice channel (V)
        const DEAFEN_MEMBERS = 1 << 23;
        /// Allows for moving of members between voice channels (V, S)
        const MOVE_MEMBERS = 1 << 24;
        /// Allows for using voice-activity-detection in a voice channel (V)
        const USE_VAD = 1 << 25;
        /// Allows for modification of own nickname
        const CHANGE_NICKNAME = 1 << 26;
        /// Allows for modification of other users nicknames
        const MANAGE_NICKNAMES = 1 << 27;
        /// Allows management and editing of roles (T, V, S)
        const MANAGE_ROLES = 1 << 28;
        /// Allows management and editing of webhooks (T, V, S)
        const MANAGE_WEBHOOKS = 1 << 29;
        /// Allows for editing and deleting emojis, stickers, and soundboard sounds created by all users
        const MANAGE_GUILD_EXPRESSIONS = 1 << 30;
        /// Allows members to use application commands, including slash commands and context menu commands. (T, V, S)
        const USE_APPLICATION_COMMANDS = 1 << 31;
        /// Allows for requesting to speak in stage channels. (This permission is under active development and may be changed or removed.) (S)
        const REQUEST_TO_SPEAK = 1 << 32;
        /// Allows for editing and deleting scheduled events created by all users (V, S)
        const MANAGE_EVENTS = 1 << 33;
        /// Allows for deleting and archiving threads, and viewing all private threads (T)
        const MANAGE_THREADS = 1 << 34;
        /// Allows for creating public and announcement threads (T)
        const CREATE_PUBLIC_THREADS = 1 << 35;
        /// Allows for creating private threads (T)
        const CREATE_PRIVATE_THREADS = 1 << 36;
        /// Allows the usage of custom stickers from other servers (T, V, S)
        const USE_EXTERNAL_STICKERS = 1 << 37;
        /// Allows for sending messages in threads (T)
        const SEND_MESSAGES_IN_THREADS = 1 << 38;
        /// Allows for using Activities (applications with the EMBEDDED flag) in a voice channel (V)
        const USE_EMBEDDED_ACTIVITIES = 1 << 39;
        /// Allows for timing out users to prevent them from sending or reacting to messages in chat and threads, and from speaking in voice and stage channels
        const MODERATE_MEMBERS = 1 << 40;
        /// Allows for viewing role subscription insights
        const VIEW_CREATOR_MONETIZATION_ANALYTICS = 1 << 41;
        /// Allows for using soundboard in a voice channel (V)
        const USE_SOUNDBOARD = 1 << 42;
        /// Allows for creating emojis, stickers, and soundboard sounds, and editing and deleting those created by the current user. Not yet available to developers, see changelog.
        const CREATE_GUILD_EXPRESSIONS = 1 << 43;
        /// Allows for creating scheduled events, and editing and deleting those created by the current user. Not yet available to developers, see changelog. (V, S)
        const CREATE_EVENTS = 1 << 44;
        /// Allows the usage of custom soundboard sounds from other servers (V)
        const USE_EXTERNAL_SOUNDS = 1 << 45;
        /// Allows sending voice messages (T, V, S)
        const SEND_VOICE_MESSAGES = 1 << 46;
    }
}

impl<'de> Deserialize<'de> for PermissionFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u64::deserialize(deserializer)?;
        Ok(PermissionFlags::from_bits_truncate(value))
    }
}
