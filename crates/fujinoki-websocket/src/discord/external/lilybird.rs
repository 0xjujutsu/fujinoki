//! Structures from @lilybird/jsx
use serde::Deserialize;

#[derive(Clone, Debug)]
pub enum EmbedType {
    Rich,
    Image,
    Video,
    Gif,
    Article,
    Link,
}

impl<'de> Deserialize<'de> for EmbedType {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(_deserializer)?;
        match s.as_str() {
            "rich" => Ok(EmbedType::Rich),
            "image" => Ok(EmbedType::Image),
            "video" => Ok(EmbedType::Video),
            "gifv" => Ok(EmbedType::Gif),
            "article" => Ok(EmbedType::Article),
            "link" => Ok(EmbedType::Link),
            _ => Err(serde::de::Error::custom(format!(
                "Invalid EmbedType: {}",
                s
            ))),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Embed {
    pub title: Option<String>,
    pub r#type: Option<EmbedType>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub timestamp: Option<String>,
    pub color: Option<u32>,
    pub footer: Option<FooterStructure>,
    pub image: Option<ImageStructure>,
    pub thumbnail: Option<ThumbnailStructure>,
    pub video: Option<VideoStructure>,
    pub provider: Option<ProviderStructure>,
    pub author: Option<AuthorStructure>,
    pub fields: Option<Vec<FieldStructure>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ThumbnailStructure {
    pub url: String,
    pub proxy_url: Option<String>,
    pub height: Option<u32>,
    pub width: Option<u32>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct VideoStructure {
    pub url: String,
    pub proxy_url: Option<String>,
    pub height: Option<u32>,
    pub width: Option<u32>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ImageStructure {
    pub url: String,
    pub proxy_url: Option<String>,
    pub height: Option<u32>,
    pub width: Option<u32>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProviderStructure {
    pub name: Option<String>,
    pub url: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AuthorStructure {
    pub name: String,
    pub url: Option<String>,
    pub icon_url: Option<String>,
    pub proxy_icon_url: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FooterStructure {
    pub text: String,
    pub icon_url: Option<String>,
    pub proxy_icon_url: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FieldStructure {
    pub name: String,
    pub value: String,
    pub inline: Option<bool>,
}
