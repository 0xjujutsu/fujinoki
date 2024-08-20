use anyhow::Result;
use bitflags::bitflags;
use serde::{Deserialize, Deserializer, Serialize};
use turbopack_binding::turbo::tasks::{self as turbo_tasks, RcStr, Vc};

pub mod command;

use crate::{
    guild::Guild,
    id::{ApplicationId, GuildId, SkuId},
    team::Team,
    user::User,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct PartialApplication {
    /// the id of the app
    pub id: ApplicationId,
    /// the application's public flags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<ApplicationFlags>,
}

#[turbo_tasks::value(shared, serialization = "custom")]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Application {
    /// the id of the app
    pub id: ApplicationId,
    /// the name of the app
    pub name: String,
    /// the icon hash of the app
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<RcStr>,
    /// the description of the app
    pub description: String,
    /// an array of rpc origin urls, if rpc is enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rcp_orgins: Option<Vec<String>>,
    /// when false only app owner can join the app's bot to guilds
    pub bot_public: bool,
    /// when true the app's bot will only join upon completion of the full
    /// oauth2 code grant flow
    pub bot_require_code_grant: bool,
    /// the url of the app's terms of service
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms_of_service_url: Option<RcStr>,
    /// the url of the app's privacy policy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy_policy_url: Option<RcStr>,
    /// partial user object containing info on the owner of the application
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<User>,
    #[deprecated = "will be removed in v11."]
    /// an empty string.
    pub summary: String,
    /// the hex encoded key for verification in interactions and the GameSDK's
    /// GetTicket
    pub verify_key: String,
    /// if the application belongs to a team, this will be a list of the members
    /// of that team
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<Team>,
    /// guild associated with the app. For example, a developer support server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    /// a partial object of the associated guild
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild: Option<Guild>,
    /// if this application is a game sold on Discord, this field will be the id
    /// of the "Game SKU" that is created, if exists
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_sku_id: Option<SkuId>,
    /// if this application is a game sold on Discord, this field will be the
    /// URL slug that links to the store page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<RcStr>,
    /// the application's default rich presence invite cover image hash
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_image: Option<RcStr>,
    /// the application's public flags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<ApplicationFlags>,
    /// an approximate count of the app's guild membership
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approximate_guild_count: Option<i32>,
    /// up to 5 tags describing the content and functionality of the application
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// settings for the application's default in-app authorization link, if
    /// enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_params: Option<InstallParams>,
    /// the application's default custom authorization link, if enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_install_url: Option<RcStr>,
    /// the application's role connection verification entry point, which when
    /// configured will render the app as a verification method in the guild
    /// role verification configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_connections_verification_url: Option<RcStr>,
}

#[turbo_tasks::value(shared)]
pub struct ApplicationVc(Vc<Application>);

bitflags! {
    #[turbo_tasks::value(shared, serialization = "custom", eq = "manual")]
    #[derive(Serialize, Default)]
    pub struct ApplicationFlags: u32 {
        /// Indicates if an app uses the Auto Moderation API
        const APPLICATION_AUTO_MODERATION_RULE_CREATE_BADGE = 1<<6;
        /// Intent required for bots in **100 or more servers** to receive `presence_update` events
        const GATEWAY_PRESENCE = 1<<12;
        /// Intent required for bots in under 100 servers to receive presence_update events, found on the **Bot** page in your app's settings
        const GATEWAY_PRESENCE_LIMITED = 1<<13;
        /// Intent required for bots in **100 or more servers** to receive member-related events like `guild_member_add`. See the list of member-related events under `GUILD_MEMBERS`
        const GATEWAY_GUILD_MEMBERS = 1<<14;
        /// Intent required for bots in under 100 servers to receive member-related events like `guild_member_add`, found on the **Bot** page in your app's settings. See the list of member-related events under `GUILD_MEMBERS`
        const GATEWAY_GUILD_MEMBERS_LIMITED = 1<<15;
        /// Indicates unusual growth of an app that prevents verification
        const VERIFICATION_PENDING_GUILD_LIMIT = 1<<16;
        /// Indicates if an app is embedded within the Discord client (currently unavailable publicly)
        const EMBEDDED = 1<<17;
        /// Intent required for bots in **100 or more servers** to receive message content
        const GATEWAY_MESSAGE_CONTENT = 1<<18;
        /// Intent required for bots in under 100 servers to receive message content, found on the Bot page in your app's settings
        const GATEWAY_MESSAGE_CONTENT_LIMITED = 1<<19;
        /// Indicates if an app has registered global application commands
        const APPLICATION_COMMAND_BADGE = 1<<23;
    }
}

impl<'de> Deserialize<'de> for ApplicationFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u32::deserialize(deserializer)?;
        Ok(ApplicationFlags::from_bits_truncate(value))
    }
}

#[turbo_tasks::value(shared, serialization = "custom", eq = "manual")]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstallParams {
    /// the scopes to add the application to the server with
    pub scopes: Vec<String>,
    /// the permissions to request for the bot role
    pub permissions: String,
}
