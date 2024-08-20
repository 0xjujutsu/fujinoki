use serde::{Deserialize, Serialize};
use turbopack_binding::turbo::tasks::{self as turbo_tasks, RcStr, TaskInput};

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Clone, Debug, Serialize, Deserialize, TaskInput, Hash)]
pub struct Embed {
    /// title of embed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<RcStr>,
    /// type of embed (always "rich" for webhook embeds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<EmbedType>,
    /// description of embed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<RcStr>,
    /// url of embed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<RcStr>,
    /// timestamp of embed content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<RcStr>,
    /// color code of the embed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<u32>,
    /// footer information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footer: Option<EmbedFooter>,
    /// image information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<EmbedImage>,
    /// thumbnail information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<EmbedThumbnail>,
    /// video information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<EmbedVideo>,
    /// provider information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<EmbedProvider>,
    /// author information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<EmbedAuthor>,
    /// fields information, max of 25
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<EmbedField>>,
}

// TODO(kijv) better deserialize and serialize
#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Clone, Debug, Serialize, Deserialize, TaskInput, Hash)]
pub enum EmbedType {
    /// generic embed rendered from embed attributes
    Rich,
    /// image embed
    Image,
    /// video embed
    Video,
    /// animated gif image embed rendered as a video embed
    Gifv,
    /// article embed
    Article,
    /// link embed
    Link,
}

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Clone, Debug, Serialize, Deserialize, TaskInput, Hash)]
pub struct EmbedThumbnail {
    /// source url of thumbnail (only supports http(s) and attachments)
    pub url: RcStr,
    /// a proxied url of the thumbnail
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_url: Option<RcStr>,
    /// height of thumbnail
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    /// width of thumbnail
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
}

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Clone, Debug, Serialize, Deserialize, TaskInput, Hash)]
pub struct EmbedVideo {
    /// source url of video
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<RcStr>,
    /// a proxied url of the video
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_url: Option<RcStr>,
    /// height of video
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    /// width of video
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
}

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Clone, Debug, Serialize, Deserialize, TaskInput, Hash)]
pub struct EmbedImage {
    /// source url of image (only supports http(s) and attachments)
    pub url: RcStr,
    /// a proxied url of the image
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_url: Option<RcStr>,
    /// height of image
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    /// width of image
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
}

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Clone, Debug, Serialize, Deserialize, TaskInput, Hash)]
pub struct EmbedProvider {
    /// name of provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<RcStr>,
    /// url of provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<RcStr>,
}

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Clone, Debug, Serialize, Deserialize, TaskInput, Hash)]
pub struct EmbedAuthor {
    /// name of author
    pub name: RcStr,
    /// url of author (only supports http(s))
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<RcStr>,
    /// url of author icon (only supports http(s) and attachments)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<RcStr>,
    /// a proxied url of author icon
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_icon_url: Option<RcStr>,
}

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Clone, Debug, Serialize, Deserialize, TaskInput, Hash)]
pub struct EmbedFooter {
    /// footer text
    pub text: RcStr,
    /// url of footer icon (only supports http(s) and attachments)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<RcStr>,
    /// a proxied url of footer icon
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_icon_url: Option<RcStr>,
}

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Clone, Debug, Serialize, Deserialize, TaskInput, Hash)]
pub struct EmbedField {
    /// name of the field
    pub name: RcStr,
    /// value of the field
    pub value: RcStr,
    /// whether or not this field should display inline
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline: Option<bool>,
}
