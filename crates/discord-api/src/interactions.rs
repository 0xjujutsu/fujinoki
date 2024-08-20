use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use turbopack_binding::turbo::{
    tasks as turbo_tasks,
    tasks::{RcStr, TaskInput},
};

use crate::{
    channel::{embed::Embed, message::MessageFlags, AllowedMentions, Attachment},
    emoji::Emoji,
    impl_deserialize_from_bits,
};

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Clone, Debug, Serialize, Deserialize, TaskInput, Hash)]
pub struct InteractionResponse {
    pub r#type: InteractionCallbackType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<InteractionCallbackData>,
}

/// * Only valid for component-based interactions
/// ** Not available for MODAL_SUBMIT and PING interactions.
/// *** Not available for APPLICATION_COMMAND_AUTOCOMPLETE and PING
/// interactions.
#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Clone, Debug, TaskInput, Hash)]
pub enum InteractionCallbackType {
    /// ACK a Ping
    Pong = 1,
    /// respond to an interaction with a message
    ChannelMessageWithSource = 4,
    /// ACK an interaction and edit a response later, the user sees a loading
    /// state
    DeferredChannelMessageWithSource = 5,
    /// for components, ACK an interaction and edit the original message later;
    /// the user does not see a loading state
    DeferredUpdateMessage = 6,
    /// for components, edit the message the component was attached to
    UpdateMessage = 7,
    /// respond to an autocomplete interaction with suggested choices
    ApplicationCommandAutocompleteResult = 8,
    /// respond to an interaction with a popup modal
    Modal = 9,
    /// respond to an interaction with an upgrade button, only available for
    /// apps with monetization enabled
    PremiumRequired = 10,
}

impl Serialize for InteractionCallbackType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(self.clone() as u32)
    }
}

impl<'de> Deserialize<'de> for InteractionCallbackType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        match value {
            1 => Ok(InteractionCallbackType::Pong),
            4 => Ok(InteractionCallbackType::ChannelMessageWithSource),
            5 => Ok(InteractionCallbackType::DeferredChannelMessageWithSource),
            6 => Ok(InteractionCallbackType::DeferredUpdateMessage),
            7 => Ok(InteractionCallbackType::UpdateMessage),
            8 => Ok(InteractionCallbackType::ApplicationCommandAutocompleteResult),
            9 => Ok(InteractionCallbackType::Modal),
            10 => Ok(InteractionCallbackType::PremiumRequired),
            _ => Err(serde::de::Error::custom(format!(
                "Invalid InteractionCallbackType value: {}",
                value
            ))),
        }
    }
}

bitflags! {
    #[turbo_tasks::value(shared, serialization = "custom", eq = "manual")]
    #[derive(Serialize, Default, TaskInput)]
    pub struct InteractionType: u8 {
        const PING = 1;
        const APPLICATION_COMMAND = 2;
        const MESSAGE_COMPONENT = 3;
        const APPLICATION_COMMAND_AUTOCOMPLETE = 4;
        const MODAL_SUBMIT = 5;
    }
}

impl_deserialize_from_bits!(InteractionType, u8);

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Clone, Debug, Deserialize, TaskInput, Hash)]
pub enum InteractionCallbackData {
    Messages(InteractionCallbackMessagesData),
    // TODO(kijv) complete remaining variants
    // Autocomplete()
    // Modal(),
}

impl Serialize for InteractionCallbackData {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            InteractionCallbackData::Messages(data) => data.serialize(serializer),
        }
    }
}

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Clone, Default, Debug, Serialize, Deserialize, TaskInput, Hash)]
pub struct InteractionCallbackMessagesData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tts: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<RcStr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeds: Option<Vec<Embed>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_mentions: Option<AllowedMentions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<MessageFlags>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<Component>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,
}

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Clone, Debug, Serialize, Deserialize, TaskInput, Hash)]
pub enum Component {
    ActionRow(ActionRow),
    Button(Button),
    // todo(kijv) more components
    // RcStrSelect(RcStrSelect),
    // TextInput(TextInput),
    // UserSelect(UserSelect),
    // RoleSelect(RoleSelect),
    // MentionableSelect(MentionableSelect),
    // ChannelSelect(ChannelSelect),
}

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Clone, Debug, Serialize, Deserialize, TaskInput, Hash)]
pub enum ComponentType {
    /// Container for other components
    ActionRow = 1,
    /// Button object
    Button = 2,
    /// Select menu for picking from defined text options
    RcStrSelect = 3,
    /// Text input object
    TextInput = 4,
    /// Select menu for users
    UserSelect = 5,
    /// Select menu for roles
    RoleSelect = 6,
    /// Select menu for mentionables (users and roles)
    MentionableSelect = 7,
    /// Select menu for channels
    ChannelSelect = 8,
}

impl Into<u32> for ComponentType {
    fn into(self) -> u32 {
        self as u32
    }
}

impl From<u32> for ComponentType {
    fn from(value: u32) -> Self {
        match value {
            1 => ComponentType::ActionRow,
            2 => ComponentType::Button,
            3 => ComponentType::RcStrSelect,
            4 => ComponentType::TextInput,
            5 => ComponentType::UserSelect,
            6 => ComponentType::RoleSelect,
            7 => ComponentType::MentionableSelect,
            8 => ComponentType::ChannelSelect,
            _ => panic!("Invalid ComponentType value: {}", value),
        }
    }
}

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Clone, Debug, Serialize, Deserialize, TaskInput, Hash)]
pub struct ActionRow {
    pub r#type: ComponentType,
    pub components: Vec<Component>,
}

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Clone, Debug, Serialize, Deserialize, TaskInput, Hash)]
pub struct Button {
    pub r#type: ComponentType,
    pub style: ButtonStyle,
    pub label: RcStr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<Emoji>,
    pub custom_id: RcStr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<RcStr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Clone, Debug, Serialize, Deserialize, TaskInput, Hash)]
pub enum ButtonStyle {
    /// blurple
    Primary = 1,
    /// grey
    Secondary = 2,
    /// green
    Success = 3,
    /// red
    Danger = 4,
    /// grey, navigates to a URL
    Link = 5,
}
