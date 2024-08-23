use std::{fmt, ops::Deref, vec};

use reqwest::Method;
use turbopack_binding::turbo::{
    tasks as turbo_tasks,
    tasks::{RcStr, TaskInput, Vc},
};

use crate::id::{
    ApplicationId, ChannelId, CommandId, GenericId, GuildId, InteractionId, MessageId, RoleId,
    StickerId, UserId, WebhookId,
};

#[turbo_tasks::value(shared)]
#[derive(Clone, Debug, Default, TaskInput, Hash)]
pub enum UserIdOrMe {
    UserId(UserId),
    #[default]
    Me,
}

impl fmt::Display for UserIdOrMe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                UserIdOrMe::UserId(user_id) => user_id.to_string(),
                UserIdOrMe::Me => "@me".to_string(),
            }
        )
    }
}

#[turbo_tasks::value(shared)]
#[derive(Clone, Debug, Default, TaskInput, Hash)]
pub enum MessageIdOrOriginal {
    MessageId(MessageId),
    #[default]
    Original,
}

impl fmt::Display for MessageIdOrOriginal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MessageIdOrOriginal::MessageId(message_id) => message_id.to_string(),
                MessageIdOrOriginal::Original => "@original".to_string(),
            }
        )
    }
}

#[turbo_tasks::value(shared)]
#[derive(Clone, Debug, TaskInput, Hash)]
pub enum WebhookPlatform {
    Github,
    Slack,
}

impl fmt::Display for WebhookPlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                WebhookPlatform::Github => "github".to_string(),
                WebhookPlatform::Slack => "slack".to_string(),
            }
        )
    }
}

#[turbo_tasks::value(shared)]
#[derive(Clone, Debug, TaskInput, Hash)]
pub enum ArchivedStatus {
    Private,
    Public,
}

#[turbo_tasks::value(shared)]
#[derive(Clone, Debug, TaskInput, Hash)]
pub enum WebhookOrApplicationId {
    WebhookId(WebhookId),
    ApplicationId(ApplicationId),
}

impl fmt::Display for WebhookOrApplicationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                WebhookOrApplicationId::WebhookId(id) => id.to_string(),
                WebhookOrApplicationId::ApplicationId(id) => id.to_string(),
            }
        )
    }
}

impl From<WebhookId> for WebhookOrApplicationId {
    fn from(webhook_id: WebhookId) -> Self {
        WebhookOrApplicationId::WebhookId(webhook_id)
    }
}

impl From<ApplicationId> for WebhookOrApplicationId {
    fn from(application_id: ApplicationId) -> Self {
        WebhookOrApplicationId::ApplicationId(application_id)
    }
}

pub struct Routes;

#[turbo_tasks::value(shared)]
pub struct Route {
    pub endpoint: RcStr,
    pub methods: Vec<RcStr>,
}

impl Route {
    pub fn new(endpoint: String) -> Route {
        Route {
            // easier than rewriting everything
            endpoint: endpoint.into(),
            methods: vec![],
        }
    }

    pub fn add_methods(mut self, methods: Vec<Method>) -> Route {
        self.methods = methods
            .iter()
            .map(|method| method.to_string().into())
            .collect();
        self
    }

    pub fn get(mut self) -> Route {
        self.methods.push(Method::GET.to_string().into());
        self
    }

    pub fn post(mut self) -> Route {
        self.methods.push(Method::POST.to_string().into());
        self
    }

    pub fn patch(mut self) -> Route {
        self.methods.push(Method::PATCH.to_string().into());
        self
    }

    pub fn delete(mut self) -> Route {
        self.methods.push(Method::DELETE.to_string().into());
        self
    }

    pub fn put(mut self) -> Route {
        self.methods.push(Method::PUT.to_string().into());
        self
    }
}

#[turbo_tasks::value_impl]
impl Route {
    #[turbo_tasks::function]
    pub async fn endpoint(self: Vc<Self>) -> Vc<RcStr> {
        Vc::cell(self.await.unwrap().deref().endpoint.clone())
    }

    #[turbo_tasks::function]
    pub async fn methods(self: Vc<Self>) -> Vc<Vec<RcStr>> {
        Vc::cell(self.await.unwrap().deref().methods.clone())
    }
}

#[turbo_tasks::value_impl]
impl Routes {
    /// Route for:
    /// - GET `/applications/{application.id}/role-connections/metadata`
    /// - PUT `/applications/{application.id}/role-connections/metadata`
    #[turbo_tasks::function]
    pub fn application_role_connection_metadata(application_id: ApplicationId) -> Vc<Route> {
        Route::new(format!(
            "/applications/{application_id}/role-connections/metadata"
        ))
        .get()
        .put()
        .cell()
    }

    /// Route for:
    /// - GET `/guilds/{guild.id}/auto-moderation/rules`
    /// - POST `/guilds/{guild.id}/auto-moderation/rules`
    #[turbo_tasks::function]
    pub fn guild_auto_moderation_rules(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/auto-moderation/rules"))
            .get()
            .post()
            .cell()
    }

    /// Routes for:
    /// - GET    `/guilds/{guild.id}/auto-moderation/rules/{rule.id}`
    /// - PATCH  `/guilds/{guild.id}/auto-moderation/rules/{rule.id}`
    /// - DELETE `/guilds/{guild.id}/auto-moderation/rules/{rule.id}`
    #[turbo_tasks::function]
    pub fn guild_auto_moderation_rule(guild_id: GuildId, rule_id: GenericId) -> Vc<Route> {
        Route::new(format!(
            "/guilds/{guild_id}/auto-moderation/rules/{rule_id}"
        ))
        .get()
        .patch()
        .delete()
        .cell()
    }

    /// Route for:
    /// - GET `/guilds/{guild.id}/audit-logs`
    #[turbo_tasks::function]
    pub fn guild_audit_log(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/audit-logs"))
            .get()
            .cell()
    }

    /// Route for:
    /// - GET    `/channels/{channel.id}`
    /// - PATCH  `/channels/{channel.id}`
    /// - DELETE `/channels/{channel.id}`
    #[turbo_tasks::function]
    pub fn channel(channel_id: ChannelId) -> Vc<Route> {
        Route::new(format!("/channels/{channel_id}"))
            .get()
            .patch()
            .delete()
            .cell()
    }

    /// Route for:
    /// - GET    `/channels/{channel.id}/messages`
    /// - POST `/channels/{channel.id}/messages`
    #[turbo_tasks::function]
    pub fn channel_messages(channel_id: ChannelId) -> Vc<Route> {
        Route::new(format!("/channels/{channel_id}/messages"))
            .get()
            .post()
            .cell()
    }

    /// Route for:
    /// - POST `/channels/{channel.id}/messages/{message.id}/crosspost`
    #[turbo_tasks::function]
    pub fn channel_message_crosspost(channel_id: ChannelId, message_id: MessageId) -> Vc<Route> {
        Route::new(format!(
            "/channels/{channel_id}/messages/{message_id}/crosspost"
        ))
        .post()
        .cell()
    }

    /// Route for:
    /// - PUT    `/channels/{channel.id}/messages/{message.id}/reactions/
    ///   {emoji}/@me`
    /// - DELETE `/channels/{channel.id}/messages/{message.id}/reactions/
    ///   {emoji}/@me`
    ///
    /// **Note**: You need to URL encode the emoji yourself
    #[turbo_tasks::function]
    pub fn channel_message_own_reaction(
        channel_id: ChannelId,
        message_id: MessageId,
        emoji: RcStr,
    ) -> Vc<Route> {
        Route::new(format!(
            "/channels/{channel_id}/messages/{message_id}/reactions/{emoji}/@me"
        ))
        .put()
        .delete()
        .cell()
    }

    /// Route for:
    /// - DELETE `/channels/{channel.id}/messages/{message.id}/reactions/
    ///   {emoji}/{user.id}`
    ///
    /// **Note**: You need to URL encode the emoji yourself
    #[turbo_tasks::function]
    pub fn channel_message_user_reaction(
        channel_id: ChannelId,
        message_id: MessageId,
        emoji: RcStr,
        user_id: UserId,
    ) -> Vc<Route> {
        Route::new(format!(
            "/channels/{channel_id}/messages/{message_id}/reactions/{emoji}/{user_id}"
        ))
        .delete()
        .cell()
    }

    /// Route for:
    /// - GET    `/channels/{channel.id}/messages/{message.id}/reactions/
    ///   {emoji}`
    /// - DELETE `/channels/{channel.id}/messages/{message.id}/reactions/
    ///   {emoji}`
    ///
    /// **Note**: You need to URL encode the emoji yourself
    #[turbo_tasks::function]
    pub fn channel_message_reaction(
        channel_id: ChannelId,
        message_id: MessageId,
        emoji: RcStr,
    ) -> Vc<Route> {
        Route::new(format!(
            "/channels/{channel_id}/messages/{message_id}/reactions/{emoji}"
        ))
        .get()
        .delete()
        .cell()
    }

    /// Route for:
    /// - DELETE `/channels/{channel.id}/messages/{message.id}/reactions`
    #[turbo_tasks::function]
    pub fn channel_message_all_reactions(
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> Vc<Route> {
        Route::new(format!(
            "/channels/{channel_id}/messages/{message_id}/reactions"
        ))
        .delete()
        .cell()
    }

    /// Route for:
    /// - POST `/channels/{channel.id}/messages/bulk-delete`
    #[turbo_tasks::function]
    pub fn channel_bulk_delete(channel_id: ChannelId) -> Vc<Route> {
        Route::new(format!("/channels/{channel_id}/messages/bulk-delete"))
            .post()
            .cell()
    }

    /// Route for:
    /// - PUT    `/channels/{channel.id}/permissions/{overwrite.id}`
    /// - DELETE `/channels/{channel.id}/permissions/{overwrite.id}`
    #[turbo_tasks::function]
    pub fn channel_permission(channel_id: ChannelId, overwrite_id: GenericId) -> Vc<Route> {
        Route::new(format!("/channels/{channel_id}/permissions/{overwrite_id}"))
            .put()
            .delete()
            .cell()
    }

    /// Route for:
    /// - GET  `/channels/{channel.id}/invites`
    /// - POST `/channels/{channel.id}/invites`
    #[turbo_tasks::function]
    pub fn channel_invites(channel_id: ChannelId) -> Vc<Route> {
        Route::new(format!("/channels/{channel_id}/invites"))
            .get()
            .post()
            .cell()
    }

    /// Route for:
    /// - POST `/channels/{channel.id}/followers`
    #[turbo_tasks::function]
    pub fn channel_followers(channel_id: ChannelId) -> Vc<Route> {
        Route::new(format!("/channels/{channel_id}/followers"))
            .post()
            .cell()
    }

    /// Route for:
    /// - POST `/channels/{channel.id}/typing`
    #[turbo_tasks::function]
    pub fn channel_typing(channel_id: ChannelId) -> Vc<Route> {
        Route::new(format!("/channels/{channel_id}/typing"))
            .post()
            .cell()
    }

    /// Route for:
    /// - GET `/channels/{channel.id}/pins`
    #[turbo_tasks::function]
    pub fn channel_pins(channel_id: ChannelId) -> Vc<Route> {
        Route::new(format!("/channels/{channel_id}/pins"))
            .get()
            .cell()
    }

    /// Route for:
    /// - PUT    `/channels/{channel.id}/pins/{message.id}`
    /// - DELETE `/channels/{channel.id}/pins/{message.id}`
    #[turbo_tasks::function]
    pub fn channel_pin(channel_id: ChannelId, message_id: MessageId) -> Vc<Route> {
        Route::new(format!("/channels/{channel_id}/pins/{message_id}"))
            .put()
            .delete()
            .cell()
    }

    /// Route for:
    /// - PUT    `/channels/{channel.id}/recipients/{user.id}`
    /// - DELETE `/channels/{channel.id}/recipients/{user.id}`
    #[turbo_tasks::function]
    pub fn channel_recipient(channel_id: ChannelId, user_id: UserId) -> Vc<Route> {
        Route::new(format!("/channels/{channel_id}/recipients/{user_id}"))
            .put()
            .delete()
            .cell()
    }

    /// Route for:
    /// - GET  `/guilds/{guild.id}/emojis`
    /// - POST `/guilds/{guild.id}/emojis`
    #[turbo_tasks::function]
    pub fn guild_emojis(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/emojis"))
            .get()
            .post()
            .cell()
    }

    /// Route for:
    /// - GET    `/guilds/{guild.id}/emojis/{emoji.id}`
    /// - PATCH  `/guilds/{guild.id}/emojis/{emoji.id}`
    /// - DELETE `/guilds/{guild.id}/emojis/{emoji.id}`
    #[turbo_tasks::function]
    pub fn guild_emoji(guild_id: GuildId, emoji_id: GenericId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/emojis/{emoji_id}"))
            .get()
            .patch()
            .delete()
            .cell()
    }

    /// Route for:
    /// - POST `/guilds`
    #[turbo_tasks::function]
    pub fn guilds() -> Vc<Route> {
        Route::new(String::from("/guilds")).post().cell()
    }

    /// Route for:
    /// - GET   `/guilds/{guild.id}`
    /// - PATCH `/guilds/{guild.id}`
    /// - DELETE `/guilds/{guild.id}`
    #[turbo_tasks::function]
    pub fn guild(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/guilds/${guild_id}"))
            .get()
            .patch()
            .delete()
            .cell()
    }

    /// Route for:
    /// - GET `/guilds/{guild.id}/preview`
    #[turbo_tasks::function]
    pub fn guild_preview(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/preview"))
            .get()
            .cell()
    }

    /// Route for:
    /// - GET   `/guilds/{guild.id}/channels`
    /// - POST  `/guilds/{guild.id}/channels`
    /// - PATCH `/guilds/{guild.id}/channels`
    #[turbo_tasks::function]
    pub fn guild_channels(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/channels"))
            .post()
            .patch()
            .get()
            .cell()
    }

    /// Route for:
    /// - GET    `/guilds/{guild.id}/members/{user.id}`
    /// - PUT    `/guilds/{guild.id}/members/{user.id}`
    /// - PATCH  `/guilds/{guild.id}/members/@me`
    /// - PATCH  `/guilds/{guild.id}/members/{user.id}`
    /// - DELETE `/guilds/{guild.id}/members/{user.id}`
    #[turbo_tasks::function]
    pub fn guild_member(guild_id: GuildId, user_id: UserIdOrMe) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/members/{user_id}"))
            .add_methods(match user_id {
                UserIdOrMe::UserId(_) => {
                    vec![Method::GET, Method::PUT, Method::PATCH, Method::DELETE]
                }
                UserIdOrMe::Me => vec![Method::PATCH],
            })
            .cell()
    }

    /// Route for:
    /// - GET `/guilds/{guild_id}/members`
    #[turbo_tasks::function]
    pub fn guild_members(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/members"))
            .get()
            .cell()
    }

    /// Route for:
    /// - GET `/guilds/{guild.id}/members/search`
    #[turbo_tasks::function]
    pub fn guild_members_search(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/members/search"))
            .get()
            .cell()
    }

    /// Route for:
    /// - PATCH `/guilds/{guild.id}/members/@me/nick`
    #[turbo_tasks::function]
    pub fn guild_current_member_nickname(_guild_id: GuildId) -> Vc<Route> {
        // This is not used as a attribute because
        // turbopack_binding::turbo::tasks::value_impl throws an error when
        // there are deprecated attributes added, so we just manual place a
        // panic here
        panic!("Deprecated: use `Routes.guild_member` instead");
    }

    /// Route for:
    /// - PUT    `/guilds/{guild.id}/members/{user.id}/roles/{role.id}`
    /// - DELETE `/guilds/{guild.id}/members/{user.id}/roles/{role.id}`
    #[turbo_tasks::function]
    pub fn guild_member_role(guild_id: GuildId, member_id: UserId, role_id: RoleId) -> Vc<Route> {
        Route::new(format!(
            "/guilds/{guild_id}/members/{member_id}/roles/{role_id}"
        ))
        .put()
        .delete()
        .cell()
    }

    /// Route for:
    /// - POST `/guilds/{guild.id}/mfa`
    #[turbo_tasks::function]
    pub fn guild_mfa(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/mfa")).post().cell()
    }

    /// Route for:
    /// - GET `/guilds/{guild.id}/bans`
    #[turbo_tasks::function]
    pub fn guild_bans(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/bans")).get().cell()
    }

    /// Route for:
    /// - GET    `/guilds/{guild.id}/bans/{user.id}`
    /// - PUT    `/guilds/{guild.id}/bans/{user.id}`
    /// - DELETE `/guilds/{guild.id}/bans/{user.id}`
    #[turbo_tasks::function]
    pub fn guild_ban(guild_id: GuildId, user_id: UserId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/bans/{user_id}"))
            .get()
            .put()
            .delete()
            .cell()
    }

    /// Routes for:
    /// - GET   `/guilds/{guild.id}/roles`
    /// - POST  `/guilds/{guild.id}/roles`
    /// - PATCH `/guilds/{guild.id}/roles`
    #[turbo_tasks::function]
    pub fn guild_roles(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/roles"))
            .get()
            .post()
            .patch()
            .cell()
    }

    /// Routes for:
    /// - PATCH  `/guilds/{guild.id}/roles/{role.id}`
    /// - DELETE `/guilds/{guild.id}/roles/{role.id}`
    #[turbo_tasks::function]
    pub fn guild_role(guild_id: GuildId, role_id: RoleId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/roles/{role_id}"))
            .patch()
            .delete()
            .cell()
    }

    /// Routes for:
    /// - GET  `/guilds/{guild.id}/prune`
    /// - POST `/guilds/{guild.id}/prune`
    #[turbo_tasks::function]
    pub fn guild_prune(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/prune"))
            .get()
            .post()
            .cell()
    }

    /// Routes for:
    /// - GET `/guilds/{guild.id}/regions`
    #[turbo_tasks::function]
    pub fn guild_voice_regions(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/regions"))
            .get()
            .cell()
    }

    /// Routes for:
    /// - GET `/guilds/{guild.id}/invites`
    #[turbo_tasks::function]
    pub fn guild_invites(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/invites"))
            .get()
            .cell()
    }

    /// Routes for:
    /// - GET `/guilds/{guild.id}/integrations`
    #[turbo_tasks::function]
    pub fn guild_integrations(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/integrations"))
            .get()
            .cell()
    }

    /// Routes for:
    /// - DELETE `/guilds/{guild.id}/integrations/{integration.id}`
    #[turbo_tasks::function]
    pub fn guild_integration(guild_id: GuildId, integration_id: GenericId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/integrations/{integration_id}"))
            .delete()
            .cell()
    }

    /// Route for:
    /// - GET   `/guilds/{guild.id}/widget`
    /// - PATCH `/guilds/{guild.id}/widget`
    #[turbo_tasks::function]
    pub fn guild_widget_settings(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/widget"))
            .get()
            .patch()
            .cell()
    }

    /// Route for:
    /// - GET   `/guilds/{guild.id}/widget`
    #[turbo_tasks::function]
    pub fn guild_widget_json(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/widget.json"))
            .get()
            .cell()
    }

    /// Route for:
    /// - GET   `/guilds/{guild.id}/vanity-url`
    #[turbo_tasks::function]
    pub fn guild_vanity_url(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/vanity-url"))
            .get()
            .cell()
    }

    /// Route for:
    /// - GET   `/guilds/{guild.id}/widget.png`
    #[turbo_tasks::function]
    pub fn guild_widget_image(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/widget.png"))
            .get()
            .cell()
    }

    /// Route for:
    /// - GET    `/invites/{invite.code}`
    /// - DELETE `/invites/{invite.code}`
    #[turbo_tasks::function]
    pub fn invite(invite_code: RcStr) -> Vc<Route> {
        Route::new(format!("/invites/{invite_code}"))
            .get()
            .delete()
            .cell()
    }

    /// Route for:
    /// - GET    `/guilds/templates/{template.code}`
    /// - POST   `/guilds/templates/{template.code}`
    #[turbo_tasks::function]
    pub fn template(template_code: RcStr) -> Vc<Route> {
        Route::new(format!("/guilds/templates/{template_code}"))
            .get()
            .post()
            .cell()
    }

    /// Route for:
    /// - GET  `/guilds/{guild.id}/templates`
    /// - POST `/guilds/{guild.id}/templates`
    #[turbo_tasks::function]
    pub fn guild_templates(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/templates"))
            .get()
            .post()
            .cell()
    }

    /// Route for:
    /// - PUT    `/guilds/templates/{template.code}`
    /// - PATCH  `/guilds/templates/{template.code}`
    /// - DELETE `/guilds/templates/{template.code}`
    #[turbo_tasks::function]
    pub fn guild_template(guild_id: GuildId, template_code: RcStr) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/templates/{template_code}"))
            .put()
            .patch()
            .cell()
    }

    /// Route for:
    /// - POST `/channels/{channel.id}/threads`
    /// - POST `/channels/{channel.id}/messages/{message.id}/threads`
    #[turbo_tasks::function]
    pub fn threads(parent_id: GenericId, message_id: Option<MessageId>) -> Vc<Route> {
        Route::new(format!(
            "/channels/{parent_id}{}",
            match message_id {
                Some(message_id) => format!("/messages/{message_id}/threads"),
                None => "/threads".to_string(),
            }
        ))
        .post()
        .cell()
    }

    /// Route for:
    /// - GET `/guilds/{guild.id}/threads/active`
    #[turbo_tasks::function]
    pub fn guild_active_threads(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/threads/active"))
            .get()
            .cell()
    }

    /// Route for:
    /// - GET `/channels/{channel.id}/threads/archived/public`
    /// - GET `/channels/{channel.id}/threads/archived/private`
    #[turbo_tasks::function]
    pub fn channel_threads(channel_id: ChannelId, archived_status: ArchivedStatus) -> Vc<Route> {
        Route::new(format!(
            "/channels/{channel_id}/threads/archived/{}",
            // TODO automatically serialize and deserialize this from string
            match archived_status {
                ArchivedStatus::Private => "private",
                ArchivedStatus::Public => "public",
            }
        ))
        .get()
        .cell()
    }

    /// Route for:
    /// - GET `/channels/{channel.id}/users/@me/threads/archived/private`
    #[turbo_tasks::function]
    pub fn channel_joined_archived_threads(channel_id: ChannelId) -> Vc<Route> {
        Route::new(format!(
            "/channels/{channel_id}/users/@me/threads/archived/private"
        ))
        .get()
        .cell()
    }

    /// Route for:
    ///  - GET    `/channels/{thread.id}/thread-members`
    ///  - GET    `/channels/{thread.id}/thread-members/{user.id}`
    ///  - PUT    `/channels/{thread.id}/thread-members/@me`
    ///  - PUT    `/channels/{thread.id}/thread-members/{user.id}`
    ///  - DELETE `/channels/{thread.id}/thread-members/@me`
    ///  - DELETE `/channels/{thread.id}/thread-members/{user.id}`
    #[turbo_tasks::function]
    pub fn thread_members(thread_id: GenericId, user_id: Option<UserIdOrMe>) -> Vc<Route> {
        Route::new(format!(
            "/channels/{thread_id}/thread-members{}",
            match user_id {
                Some(user_id_or_me) => user_id_or_me.to_string(),
                None => "".to_string(),
            }
        ))
        .get()
        .put()
        .delete()
        .cell()
    }

    /// Route for:
    /// - GET   `/users/@me`
    /// - GET   `/users/{user.id}`
    /// - PATCH `/users/@me`
    #[turbo_tasks::function]
    pub fn user(user_id: UserIdOrMe) -> Vc<Route> {
        Route::new(format!("/users/{user_id}"))
            .add_methods(match user_id {
                UserIdOrMe::Me => vec![Method::GET, Method::PATCH],
                UserIdOrMe::UserId(_) => vec![Method::GET],
            })
            .cell()
    }

    /// Route for:
    /// - GET `/users/@me/applications/{application.id}/role-connection`
    /// - PUT `/users/@me/applications/{application.id}/role-connection`
    #[turbo_tasks::function]
    pub fn user_application_role_connection(application_id: ApplicationId) -> Vc<Route> {
        Route::new(format!(
            "/users/@me/applications/{application_id}/role-connection"
        ))
        .get()
        .put()
        .cell()
    }

    /// Route for:
    /// - GET `/users/@me/guilds`
    #[turbo_tasks::function]
    pub fn user_guilds() -> Vc<Route> {
        Route::new(String::from("/users/@me/guilds")).get().cell()
    }

    /// Route for:
    /// - GET `/users/@me/guilds/{guild.id}/member`
    #[turbo_tasks::function]
    pub fn user_guild_member(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/users/@me/guilds/{guild_id}/member"))
            .get()
            .cell()
    }

    /// Route for:
    /// - DELETE `/users/@me/guilds/{guild.id}`
    #[turbo_tasks::function]
    pub fn user_guild(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/users/@me/guilds/{guild_id}"))
            .delete()
            .cell()
    }

    /// Route for:
    /// - POST `/users/@me/channels`
    #[turbo_tasks::function]
    pub fn user_channels() -> Vc<Route> {
        Route::new(String::from("/users/@me/channels"))
            .post()
            .cell()
    }

    /// Route for:
    /// - GET `/users/@me/connections`
    #[turbo_tasks::function]
    pub fn user_connections() -> Vc<Route> {
        Route::new(String::from("/users/@me/connections"))
            .get()
            .cell()
    }

    /// Route for:
    /// - GET `/voice/regions`
    #[turbo_tasks::function]
    pub fn voice_regions() -> Vc<Route> {
        Route::new(String::from("/voice/regions")).get().cell()
    }

    /// Route for:
    /// - GET  `/channels/{channel.id}/webhooks`
    /// - POST `/channels/{channel.id}/webhooks`
    #[turbo_tasks::function]
    pub fn channel_webhooks(channel_id: ChannelId) -> Vc<Route> {
        Route::new(format!("/channels/{channel_id}/webhooks"))
            .get()
            .post()
            .cell()
    }

    /// Route for:
    /// - GET  `/guilds/{guild.id}/webhooks`
    #[turbo_tasks::function]
    pub fn guild_webhooks(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/webhooks"))
            .get()
            .cell()
    }

    /// Route for:
    /// - GET    `/webhooks/{webhook.id}`
    /// - GET    `/webhooks/{webhook.id}/{webhook.token}`
    /// - PATCH  `/webhooks/{webhook.id}`
    /// - PATCH  `/webhooks/{webhook.id}/{webhook.token}`
    /// - DELETE `/webhooks/{webhook.id}`
    /// - DELETE `/webhooks/{webhook.id}/{webhook.token}`
    /// - POST   `/webhooks/{webhook.id}/{webhook.token}`
    ///
    /// - POST   `/webhooks/{application.id}/{interaction.token}`
    #[turbo_tasks::function]
    pub fn webhook(id: WebhookOrApplicationId, token: Option<RcStr>) -> Vc<Route> {
        Route::new(format!(
            "/webhooks/{id}{}",
            match &token {
                Some(token) => format!("/{token}"),
                None => "".to_string(),
            }
        ))
        .add_methods(match &token {
            Some(_) => vec![Method::GET, Method::PATCH, Method::DELETE, Method::POST],
            None => vec![Method::GET, Method::PATCH, Method::DELETE],
        })
        .cell()
    }

    /// Route for:
    /// - GET    `/webhooks/{webhook.id}/{webhook.token}/messages/@original`
    /// - GET    `/webhooks/{webhook.id}/{webhook.token}/messages/{message.id}`
    /// - PATCH  `/webhooks/{webhook.id}/{webhook.token}/messages/@original`
    /// - PATCH  `/webhooks/{webhook.id}/{webhook.token}/messages/{message.id}`
    /// - DELETE `/webhooks/{webhook.id}/{webhook.token}/messages/@original`
    /// - DELETE `/webhooks/{webhook.id}/{webhook.token}/messages/{message.id}`
    ///
    /// - GET  `/webhooks/{application.id}/{interaction.token}/messages/@
    ///   original`
    /// - PATCH  `/webhooks/{application.id}/{interaction.token}/messages/@
    ///   original`
    /// - PATCH  `/webhooks/{application.id}/{interaction.token}/messages/
    ///   {message.id}`
    /// - DELETE `/webhooks/{application.id}/{interaction.token}/messages/
    ///   {message.id}`
    #[turbo_tasks::function]
    pub fn webhook_message(
        id: WebhookOrApplicationId,
        token: RcStr,
        message_id: MessageIdOrOriginal,
    ) -> Vc<Route> {
        Route::new(format!("/webhooks/{id}/{token}/messages/{message_id}"))
            .add_methods(match id {
                WebhookOrApplicationId::WebhookId(_) => {
                    vec![Method::GET, Method::PATCH, Method::DELETE]
                }
                WebhookOrApplicationId::ApplicationId(_) => {
                    vec![Method::PATCH, Method::DELETE, Method::GET]
                }
            })
            .cell()
    }

    /// Route for:
    /// - POST `/webhooks/{webhook.id}/{webhook.token}/github`
    /// - POST `/webhooks/{webhook.id}/{webhook.token}/slack`
    #[turbo_tasks::function]
    pub fn webhook_platform(
        webhook_id: WebhookId,
        webhook_token: RcStr,
        platform: WebhookPlatform,
    ) -> Vc<Route> {
        Route::new(format!("/webhooks/{webhook_id}/{webhook_token}/{platform}"))
            .post()
            .cell()
    }

    /// Route for:
    /// - GET `/gateway`
    #[turbo_tasks::function]
    pub fn gateway() -> Vc<Route> {
        Route::new(String::from("/gateway")).get().cell()
    }

    /// Route for:
    /// - GET `/gateway/bot`
    #[turbo_tasks::function]
    pub fn gateway_bot() -> Vc<Route> {
        Route::new(String::from("/gateway/bot")).get().cell()
    }

    /// Route for:
    /// - GET `/oauth2/applications/@me`
    #[turbo_tasks::function]
    pub fn oauth2_current_application() -> Vc<Route> {
        Route::new(String::from("/oauth2/applications/@me"))
            .get()
            .cell()
    }

    /// Route for:
    /// - GET `/oauth2/@me`
    #[turbo_tasks::function]
    pub fn oauth2_current_authorization() -> Vc<Route> {
        Route::new(String::from("/oauth2/@me")).get().cell()
    }

    /// Route for:
    /// - GET `/oauth2/authorize`
    #[turbo_tasks::function]
    pub fn oauth2_authorization() -> Vc<Route> {
        Route::new(String::from("/oauth2/authorize")).get().cell()
    }

    /// Route for:
    /// - POST `/oauth2/token`
    #[turbo_tasks::function]
    pub fn oauth2_token_exchange() -> Vc<Route> {
        Route::new(String::from("/oauth2/token")).post().cell()
    }

    /// Route for:
    /// - POST `/oauth2/token/revoke`
    #[turbo_tasks::function]
    pub fn oauth2_token_revocation() -> Vc<Route> {
        Route::new(String::from("/oauth2/token/revoke"))
            .post()
            .cell()
    }

    /// Route for:
    /// - GET  `/applications/{application.id}/commands`
    /// - PUT  `/applications/{application.id}/commands`
    /// - POST `/applications/{application.id}/commands`
    #[turbo_tasks::function]
    pub fn application_commands(application_id: ApplicationId) -> Vc<Route> {
        Route::new(format!("/applications/{application_id}/commands"))
            .get()
            .put()
            .post()
            .cell()
    }

    /// Route for:
    /// - GET    `/applications/{application.id}/commands/{command.id}`
    /// - PATCH  `/applications/{application.id}/commands/{command.id}`
    /// - DELETE `/applications/{application.id}/commands/{command.id}`
    #[turbo_tasks::function]
    pub fn application_command(application_id: ApplicationId, command_id: CommandId) -> Vc<Route> {
        Route::new(format!(
            "/applications/{application_id}/commands/{command_id}"
        ))
        .get()
        .patch()
        .delete()
        .cell()
    }

    /// Route for:
    /// - GET  `/applications/{application.id}/guilds/{guild.id}/commands`
    /// - PUT  `/applications/{application.id}/guilds/{guild.id}/commands`
    /// - POST `/applications/{application.id}/guilds/{guild.id}/commands`
    #[turbo_tasks::function]
    pub fn application_guild_commands(
        application_id: ApplicationId,
        guild_id: GuildId,
    ) -> Vc<Route> {
        Route::new(format!(
            "/applications/{application_id}/guilds/{guild_id}/commands"
        ))
        .get()
        .post()
        .put()
        .cell()
    }

    /// Route for:
    /// - GET    `/applications/{application.id}/guilds/{guild.id}/commands/
    ///   {command.id}`
    /// - PATCH  `/applications/{application.id}/guilds/{guild.id}/commands/
    ///   {command.id}`
    /// - DELETE `/applications/{application.id}/guilds/{guild.id}/commands/
    ///   {command.id}`
    #[turbo_tasks::function]
    pub fn application_guild_command(
        application_id: ApplicationId,
        guild_id: GuildId,
        command_id: GenericId,
    ) -> Vc<Route> {
        Route::new(format!(
            "/applications/{application_id}/guilds/{guild_id}/commands/{command_id}"
        ))
        .get()
        .patch()
        .delete()
        .cell()
    }

    /// Route for:
    /// - POST `/interactions/{interaction.id}/{interaction.token}/callback`
    #[turbo_tasks::function]
    pub fn interaction_callback(
        interaction_id: InteractionId,
        interaction_token: RcStr,
    ) -> Vc<Route> {
        Route::new(format!(
            "/interactions/{interaction_id}/{interaction_token}/callback"
        ))
        .post()
        .cell()
    }

    /// Route for:
    /// - GET   `/guilds/{guild.id}/member-verification`
    /// - PATCH `/guilds/{guild.id}/member-verification`
    #[turbo_tasks::function]
    pub fn guild_member_verification(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/member-verification"))
            .get()
            .patch()
            .cell()
    }

    /// Route for:
    /// - PATCH `/guilds/{guild.id}/voice-states/@me`
    /// - PATCH `/guilds/{guild.id}/voice-states/{user.id}`
    #[turbo_tasks::function]
    pub fn guild_voice_state(guild_id: GuildId, user_id: Option<UserIdOrMe>) -> Vc<Route> {
        Route::new(format!(
            "/guilds/{guild_id}/voice-states/{user_id}",
            user_id = user_id.unwrap_or_default()
        ))
        .patch()
        .cell()
    }

    /// Route for:
    /// - GET `/applications/{application.id}/guilds/{guild.id}/commands/
    ///   permissions`
    /// - PUT `/applications/{application.id}/guilds/{guild.id}/commands/
    ///   permissions`
    #[turbo_tasks::function]
    pub fn guild_application_commands_permissions(
        application_id: ApplicationId,
        guild_id: GuildId,
    ) -> Vc<Route> {
        Route::new(format!(
            "/applications/{application_id}/guilds/{guild_id}/commands/permissions"
        ))
        .get()
        .put()
        .cell()
    }

    /// Route for:
    /// - GET `/applications/{application.id}/guilds/{guild.id}/commands/
    ///   {command.id}/permissions`
    /// - PUT `/applications/{application.id}/guilds/{guild.id}/commands/
    ///   {command.id}/permissions`
    #[turbo_tasks::function]
    pub fn application_command_permissions(
        application_id: ApplicationId,
        guild_id: GuildId,
        command_id: GenericId,
    ) -> Vc<Route> {
        Route::new(format!(
            "/applications/{application_id}/guilds/{guild_id}/commands/{command_id}/permissions"
        ))
        .get()
        .put()
        .cell()
    }
    /// Route for:
    /// - GET   `/guilds/{guild.id}/welcome-screen`
    /// - PATCH `/guilds/{guild.id}/welcome-screen`
    #[turbo_tasks::function]
    pub fn guild_welcome_screen(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/welcome-screen"))
            .get()
            .patch()
            .cell()
    }

    /// Route for:
    /// - POST `/stage-instances`
    #[turbo_tasks::function]
    pub fn stage_instances() -> Vc<Route> {
        Route::new(String::from("/stage-instances")).post().cell()
    }

    /// Route for:
    /// - GET    `/stage-instances/{channel.id}`
    /// - PATCH  `/stage-instances/{channel.id}`
    /// - DELETE `/stage-instances/{channel.id}`
    #[turbo_tasks::function]
    pub fn stage_instance(channel_id: ChannelId) -> Vc<Route> {
        Route::new(format!("/stage-instances/{channel_id}")).cell()
    }

    /// Route for:
    /// - GET `/stickers/{sticker.id}`
    #[turbo_tasks::function]
    pub fn sticker(sticker_id: StickerId) -> Vc<Route> {
        Route::new(format!("/stickers/{sticker_id}")).get().cell()
    }

    /// Route for:
    /// - GET `/sticker-packs`
    #[turbo_tasks::function]
    pub fn sticker_packs() -> Vc<Route> {
        Route::new(String::from("/sticker-packs")).get().cell()
    }

    /// Route for:
    /// - GET `/sticker-packs`
    #[turbo_tasks::function]
    pub fn nitro_sticker_packs() -> Vc<Route> {
        panic!("Deprecated: use `Routes.sticker_packs` instead");

        #[allow(unreachable_code)]
        Route::new(String::from("/sticker-packs")).get().cell()
    }

    /// Route for:
    /// - GET  `/guilds/{guild.id}/stickers`
    /// - POST `/guilds/{guild.id}/stickers`
    #[turbo_tasks::function]
    pub fn guild_stickers(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/stickers"))
            .get()
            .post()
            .cell()
    }

    /// Route for:
    /// - GET    `/guilds/{guild.id}/stickers/{sticker.id}`
    /// - PATCH  `/guilds/{guild.id}/stickers/{sticker.id}`
    /// - DELETE `/guilds/{guild.id}/stickers/{sticker.id}`
    #[turbo_tasks::function]
    pub fn guild_sticker(guild_id: GuildId, sticker_id: StickerId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/stickers/{sticker_id}"))
            .get()
            .patch()
            .delete()
            .cell()
    }

    /// Route for:
    /// - GET  `/guilds/{guild.id}/scheduled-events`
    /// - POST `/guilds/{guild.id}/scheduled-events`
    #[turbo_tasks::function]
    pub fn guild_scheduled_events(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/scheduled-events"))
            .get()
            .post()
            .cell()
    }

    /// Route for:
    /// - GET    `/guilds/{guild.id}/scheduled-events/{guildScheduledEvent.id}`
    /// - PATCH  `/guilds/{guild.id}/scheduled-events/{guildScheduledEvent.id}`
    /// - DELETE `/guilds/{guild.id}/scheduled-events/{guildScheduledEvent.id}`
    #[turbo_tasks::function]
    pub fn guild_scheduled_event(
        guild_id: GuildId,
        guild_scheduled_event_id: GenericId,
    ) -> Vc<Route> {
        Route::new(format!(
            "/guilds/{guild_id}/scheduled-events/{guild_scheduled_event_id}"
        ))
        .get()
        .patch()
        .delete()
        .cell()
    }

    /// Route for:
    /// - GET `/guilds/{guild.id}/scheduled-events/{guildScheduledEvent.id}/
    ///   users`
    #[turbo_tasks::function]
    pub fn guild_scheduled_event_users(
        guild_id: GuildId,
        guild_scheduled_event_id: GenericId,
    ) -> Vc<Route> {
        Route::new(format!(
            "/guilds/{guild_id}/scheduled-events/{guild_scheduled_event_id}/users"
        ))
        .get()
        .cell()
    }

    /// Route for:
    /// - GET `/guilds/${guild.id}/onboarding`
    /// - PUT `/guilds/${guild.id}/onboarding`
    #[turbo_tasks::function]
    pub fn guild_onboarding(guild_id: GuildId) -> Vc<Route> {
        Route::new(format!("/guilds/{guild_id}/onboarding"))
            .get()
            .put()
            .cell()
    }

    /// Route for:
    ///  - GET `/applications/@me`
    #[turbo_tasks::function]
    pub fn current_application() -> Vc<Route> {
        Route::new(String::from("/applications/@me")).get().cell()
    }
}
