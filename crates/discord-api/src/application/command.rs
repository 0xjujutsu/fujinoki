use lazy_regex::{lazy_regex, Lazy};
use regex::Regex;
use serde::{Deserialize, Serialize};
use turbopack_binding::turbo::tasks as turbo_tasks;

use crate::{
    channel::ChannelTypes,
    id::{ApplicationId, CommandId, GenericId, GuildId},
    permissions::PermissionFlags,
};

pub static CHAT_INPUT_NAME: Lazy<Regex> =
    lazy_regex!(r"^[-_\p{L}\p{N}\p{sc=Deva}\p{sc=Thai}]{1,32}$");

fn some_true() -> Option<bool> {
    Some(true)
}

fn some_false() -> Option<bool> {
    Some(false)
}

#[derive(Debug, Clone, PartialEq)]
pub enum IntegerOrDouble {
    Integer(u32),
    Double(f32),
}

impl<'se> serde::Serialize for IntegerOrDouble {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self {
            IntegerOrDouble::Integer(v) => serializer.serialize_u32(*v),
            IntegerOrDouble::Double(v) => serializer.serialize_f32(*v),
        }
    }
}

impl<'de> serde::Deserialize<'de> for IntegerOrDouble {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let float = f32::deserialize(deserializer)?;
        if float.fract() == 0.0 {
            Ok(IntegerOrDouble::Integer(float as u32))
        } else {
            Ok(IntegerOrDouble::Double(float))
        }
    }
}

#[turbo_tasks::value(shared, serialization = "custom", eq = "manual")]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct ApplicationCommand {
    pub id: CommandId,
    pub r#type: ApplicationCommandType,
    pub application_id: ApplicationId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    pub name: String,
    // name_localizations: Locales,
    pub description: String,
    // description_localizations: Locales,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<ApplicationCommandOption>>,
    /// `0` to disable the command for everyone except admins by default,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_member_permissions: Option<PermissionFlags>,
    /// Indicates whether the command is available in DMs with the app, only for
    /// globally-scoped commands. By default, commands are visible.
    #[deprecated = "use contexts instead"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dm_permission: Option<bool>,
    #[deprecated(note = "Not recommended for use as field will soon be deprecated.")]
    #[serde(skip_serializing_if = "Option::is_none", default = "some_true")]
    pub default_permission: Option<bool>,
    /// [In preview](https://discord.com/developers/docs/change-log#userinstallable-apps-preview). [Installation context(s)](https://discord.com/developers/docs/resources/application#installation-context) where the command is available, only for globally-scoped commands. Defaults to `GUILD_INSTALL` (0)
    #[serde(
        skip_serializing_if = "Option::is_none",
        default = "ApplicationIntegration::default"
    )]
    integration_type: Option<Vec<ApplicationIntegration>>,
    /// [In preview](https://discord.com/developers/docs/change-log#userinstallable-apps-preview). [Installation context(s)](https://discord.com/developers/docs/resources/application#installation-context) where the command can be used, only for globally-scoped commands. By default, all interaction context types included for new commands.
    #[serde(
        skip_serializing_if = "Option::is_none",
        default = "InteractionContext::all"
    )]
    contexts: Option<Vec<InteractionContext>>,
    /// Indicates whether the command is [age-restricted](https://discord.com/developers/docs/interactions/application-commands#agerestricted-commands), defaults to `false``
    #[serde(skip_serializing_if = "Option::is_none", default = "some_false")]
    pub nsfw: Option<bool>,
    pub version: GenericId,
}

#[turbo_tasks::value(shared, serialization = "custom", eq = "manual")]
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ApplicationCommandType {
    #[default]
    ChatInput = 1,
    User = 2,
    Message = 3,
}

impl<'de> serde::Deserialize<'de> for ApplicationCommandType {
    fn deserialize<D>(deserializer: D) -> Result<ApplicationCommandType, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u32::deserialize(deserializer)?;
        Ok(match value {
            1 => ApplicationCommandType::ChatInput,
            2 => ApplicationCommandType::User,
            3 => ApplicationCommandType::Message,
            _ => ApplicationCommandType::ChatInput,
        })
    }
}

impl<'se> serde::Serialize for ApplicationCommandType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value = match self {
            ApplicationCommandType::ChatInput => 1,
            ApplicationCommandType::User => 2,
            ApplicationCommandType::Message => 3,
        };
        serializer.serialize_u32(value)
    }
}

#[turbo_tasks::value(shared, serialization = "custom", eq = "manual")]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct ApplicationCommandOption {
    pub r#type: ApplicationCommandOptionType,
    pub name: String,
    // name_localizations: Locales,
    pub description: String,
    // description_localizations: Locales,
    #[serde(skip_serializing_if = "Option::is_none", default = "some_false")]
    pub required: Option<bool>,
    pub choices: Vec<ApplicationCommandOptionChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<ApplicationCommandOption>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_types: Option<ChannelTypes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_value: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_value: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default = "some_false")]
    pub autocomplete: Option<bool>,
}

#[turbo_tasks::value(shared, serialization = "custom", eq = "manual")]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum ApplicationCommandOptionType {
    SubCommand = 1,
    SubCommandGroup = 2,
    String = 3,
    Integer = 4,
    Boolean = 5,
    User = 6,
    /// Includes all channel types + categories
    Channel = 7,
    Role = 8,
    /// Includes users and roles
    Mentionable = 9,
    /// Any double between -2^53 and 2^53
    Number = 10,
    /// [Attachment] object
    Attachment = 11,
}

#[turbo_tasks::value(shared, serialization = "custom", eq = "manual")]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct ApplicationCommandOptionChoice {
    name: String,
    // name_localizations: Locales,
    value: ApplicationCommandOptionChoiceValue,
}

#[turbo_tasks::value(shared, serialization = "custom", eq = "manual")]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
enum ApplicationCommandOptionChoiceValue {
    String(String),
    Integer(u32),
    Double(f64),
}

#[turbo_tasks::value(shared, serialization = "custom", eq = "manual")]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum ApplicationIntegration {
    /// App is installable to servers
    GuildInstall = 0,
    /// App is installable to users
    UserInstall = 1,
}

impl ApplicationIntegration {
    pub fn default() -> Option<Vec<ApplicationIntegration>> {
        Some(vec![ApplicationIntegration::GuildInstall])
    }
}

#[turbo_tasks::value(shared, serialization = "custom", eq = "manual")]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum InteractionContext {
    /// Interaction can be used within servers
    Guild = 0,
    /// Interaction can be used within DMs with the app's bot user
    BotDm = 1,
    /// Interaction can be used within Group DMs and DMs other than the app's
    /// bot user
    PrivateChannel = 2,
}

impl InteractionContext {
    pub fn all() -> Option<Vec<InteractionContext>> {
        Some(vec![
            InteractionContext::Guild,
            InteractionContext::BotDm,
            InteractionContext::PrivateChannel,
        ])
    }
}
