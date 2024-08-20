use std::{collections::BTreeMap, sync::Arc};

use anyhow::{Context, Result};
use discord_api::{
    channel::embed::{
        Embed, EmbedAuthor, EmbedField, EmbedFooter, EmbedImage, EmbedProvider, EmbedThumbnail,
        EmbedType, EmbedVideo,
    },
    gateway::{OpCodeName, Payload, ReadyEventPayload},
    id::InteractionId,
    interactions::{
        InteractionCallbackData, InteractionCallbackMessagesData, InteractionCallbackType,
        InteractionResponse,
    },
};
use serde_json::{json, Value as JsonValue};
use tokio::sync::Mutex;
use turbopack_binding::{
    turbo::{
        tasks as turbo_tasks,
        tasks::{run_once_with_reason, RcStr, ValueToString, Vc},
        tasks_bytes::stream::SingleValue,
        tasks_fs::{json::parse_json_with_source_context, FileSystemPath},
    },
    turbopack::{
        core::{
            file_source::FileSource,
            issue::{handle_issues, IssueExt, IssueReporter, IssueSeverity},
            module::{Module, OptionModule},
            source::Source,
        },
        node::evaluate::evaluate,
    },
};

use super::{external::lilybird, issue::RuntimeIssue};
use crate::{
    invalidation::WebsocketMessageSideEffects,
    source::{ContentSource, ContentSourceData},
    util::CamelCaseJson,
    WebsocketContext,
};

// TODO break each event into its own function if possible
#[turbo_tasks::function]
pub async fn dispatch(
    json: Vc<Payload>,
    source: Vc<ContentSourceData>,
    issue_reporter: Vc<Box<dyn IssueReporter>>,
    ctx: Vc<WebsocketContext>,
) -> Result<Vc<()>> {
    let json = &*json.await?;

    // keeping this here just in case: but I don't think any issues are emitted
    // (this takes ~150ms)
    handle_issues(
        source,
        issue_reporter,
        IssueSeverity::Fatal.cell(),
        Some(
            &json
                .clone()
                .t
                .unwrap_or(json.clone().op.name().to_string().into()),
        ),
        Some("get source"),
    )
    .await?;

    let mut data = json.clone().d.expect("failed to retrieve dispatch data");
    let data = data.as_object_mut().unwrap();

    let event_name: RcStr = json.clone().t.unwrap().into();

    let side_effects_reason = WebsocketMessageSideEffects {
        opcode: json.op,
        event: json.t.as_ref().map(|t| t.to_string()),
    };

    // TODO make executing user-events a function for reusability and less
    // redundancy
    match &*event_name.clone().to_string() {
        "READY" => {
            let ctx = ctx.await?;
            let resolved_source = source.resolve_strongly_consistent().await?;

            let mut clean_client_data = ctx
                .clean_client_data
                .try_lock()
                .expect("failed to lock `clean_client_data`");

            let mut session_id = ctx
                .session_id
                .try_lock()
                .expect("failed to lock `session_id`");
            let mut resume_gateway_url = ctx
                .resume_gateway_url
                .try_lock()
                .expect("failed to lock `resume_gateway_url`");

            *session_id = Some(data["session_id"].as_str().unwrap().to_string());
            *resume_gateway_url = Some(data["resume_gateway_url"].as_str().unwrap().to_string());

            // Removes unnecessary data from the ready event, which will be used when
            // calling the ready event handler
            let mut client_data = data.clone();
            client_data.remove("_trace");
            client_data.remove("geo_ordered_rtc_regions");
            client_data.remove("session_id");
            client_data.remove("resume_gateway_url");

            // * this should be faster once Turbo engine has persistent caching
            // if resolved_source.get_commands_dir().await?.is_some() {
            //     let application_id: ApplicationId = client_data["application"]["id"]
            //         .as_str()
            //         .unwrap()
            //         .parse()
            //         .unwrap();
            //     util::update_application_commands(
            //         application_id,
            //         resolved_source,
            //         resolved_source.get_commands(),
            //         resolved_source.get_entries(),
            //         ctx.config.clone(),
            //         issue_reporter.clone(),
            //         Vc::cell(Some(
            //             resolved_source
            //                 .get_commands_dir()
            //                 .routes_changed(ctx.config),
            //         )),
            //     )
            //     .await?;
            // }

            let client_data = ReadyEventPayload {
                client: JsonValue::Object(client_data.camel_case_json()),
            };

            *clean_client_data = Some(client_data.client.clone());
            drop(clean_client_data);

            if let Some(entry) = get_event_entry(resolved_source, event_name.clone())
                .await?
                .clone_value()
            {
                let debug = ctx.debug;
                let config = ctx.config;

                let join_handle = tokio::spawn(run_once_with_reason(
                    ctx.turbo_tasks.clone(),
                    side_effects_reason,
                    async move {
                        let project_path = resolved_source.clone().await?.project_path;
                        let chunking_context = resolved_source
                            .clone()
                            .await?
                            .executor
                            .await?
                            .chunking_context;
                        let asset_context =
                            resolved_source.clone().await?.executor.await?.asset_context;
                        let env = resolved_source.clone().await?.executor.await?.env;

                        let evaluated = evaluate(
                            entry,
                            project_path,
                            env,
                            FileSource::new(project_path).ident(),
                            asset_context,
                            Vc::upcast(chunking_context),
                            None,
                            vec![Vc::cell(serde_json::to_value(client_data).unwrap())],
                            resolved_source.get_events_dir().routes_changed(config),
                            debug,
                        );

                        handle_issues(
                            evaluated,
                            issue_reporter,
                            IssueSeverity::Fatal.cell(),
                            None,
                            Some("evaluate js"),
                        )
                        .await?;

                        Ok(())
                    },
                ));
                ctx.ongoing_side_effects
                    .lock()
                    .await
                    .push_back(Arc::new(Mutex::new(Some(join_handle))));
            };

            Ok(Default::default())
        }
        "INTERACTION_CREATE" => {
            let resolved_source = source.resolve_strongly_consistent().await?;
            let commands = resolved_source.get_commands().await?;
            let commands: std::collections::BTreeMap<RcStr, Vc<FileSystemPath>> = commands
                .iter()
                .map(|v| (v.name.clone(), v.file_path.clone()))
                .collect::<BTreeMap<_, _>>();

            let command_name: &RcStr = &data["data"]["name"].as_str().unwrap().to_string().into();
            let command_data = commands.get(command_name);
            let entries = resolved_source.clone().get_entries();
            let command = if let Some(file_path) = command_data {
                *entries.get_entry(file_path.to_string()).await?
            } else {
                None
            };

            // TODO abstract duplicate code

            let event = get_event_entry(resolved_source, event_name.clone());
            // User-provided event
            if let Some(event_entry) = *event.await? {
                let ctx = ctx.await?;
                let data = data.clone();
                let clean_client_data = ctx.clean_client_data.clone();
                let debug = ctx.debug;
                let config = ctx.config;

                let join_handle = tokio::spawn(run_once_with_reason(
                    ctx.turbo_tasks.clone(),
                    side_effects_reason.clone(),
                    async move {
                        let project_path = resolved_source.clone().await?.project_path;
                        let chunking_context = resolved_source
                            .clone()
                            .await?
                            .executor
                            .await?
                            .chunking_context;
                        let asset_context =
                            resolved_source.clone().await?.executor.await?.asset_context;
                        let env = resolved_source.clone().await?.executor.await?.env;

                        let clean_client_data = clean_client_data
                            .try_lock()
                            .expect("failed to lock `clean_client_data`")
                            .clone()
                            .unwrap_or(JsonValue::Object(serde_json::Map::default()));

                        // TODO(kijv) only update this application_command (route_changed for
                        // invalidation) if resolved_source.await?.commands_dir.await?.
                        // clone_value().is_some() {     let application_id:
                        // ApplicationId = clean_client_data["application"]["id"].
                        // as_str().unwrap().parse().unwrap();
                        //     util::update_application_commands(
                        //         application_id,
                        //         resolved_source,
                        //         resolved_source.get_commands(),
                        //         resolved_source.get_entries(),
                        //         config.clone(),
                        //         issue_reporter.clone(),
                        //         Vc::cell(Some(resolved_source.get_commands_dir().
                        // routes_changed(config))),     ).await?;
                        // }

                        let initial_val = evaluate(
                            event_entry,
                            project_path,
                            env,
                            FileSource::new(project_path).ident(),
                            asset_context,
                            Vc::upcast(chunking_context),
                            None,
                            vec![Vc::cell(json!({
                                "interaction": data.clone(),
                                "client": clean_client_data
                            }))],
                            resolved_source.get_events_dir().routes_changed(config),
                            debug,
                        );

                        let SingleValue::Single(_) = initial_val.await?.try_into_single().await?
                        else {
                            // An error happened, which has already been converted into an issue.
                            return handle_issues(
                                initial_val,
                                issue_reporter,
                                IssueSeverity::Fatal.cell(),
                                None,
                                None,
                            )
                            .await;
                        };

                        Ok(())
                    },
                ));
                ctx.ongoing_side_effects
                    .lock()
                    .await
                    .push_back(Arc::new(Mutex::new(Some(join_handle))));
            }

            // TODO(kijv) allow users to disable this?
            // Our own event handler
            if let Some(entry) = command {
                let ctx = ctx.await?;
                let data = data.clone();
                let clean_client_data = ctx.clean_client_data.clone();
                let debug = ctx.debug;
                let config = ctx.config;

                let join_handle = tokio::spawn(run_once_with_reason(
                    ctx.turbo_tasks.clone(),
                    side_effects_reason.clone(),
                    async move {
                        let project_path = resolved_source.clone().await?.project_path;
                        let chunking_context = resolved_source
                            .clone()
                            .await?
                            .executor
                            .await?
                            .chunking_context;
                        let asset_context =
                            resolved_source.clone().await?.executor.await?.asset_context;
                        let env = resolved_source.clone().await?.executor.await?.env;

                        let clean_client_data = clean_client_data
                            .try_lock()
                            .expect("failed to lock `clean_client_data`")
                            .clone()
                            .unwrap_or(JsonValue::Object(serde_json::Map::default()));

                        let initial_val = evaluate(
                            entry,
                            project_path,
                            env,
                            FileSource::new(project_path).ident(),
                            asset_context,
                            Vc::upcast(chunking_context),
                            None,
                            vec![Vc::cell(json!({
                                "interaction": data.clone(),
                                "client": clean_client_data
                            }))],
                            resolved_source.get_commands_dir().routes_changed(config),
                            debug,
                        );

                        let SingleValue::Single(val) = initial_val.await?.try_into_single().await?
                        else {
                            // An error happened, which has already been converted into an issue.
                            handle_issues(
                                initial_val,
                                issue_reporter,
                                IssueSeverity::Fatal.cell(),
                                None,
                                None,
                            )
                            .await?;
                            return Ok(Default::default());
                        };
                        let initial_return: JsonValue =
                            parse_json_with_source_context(val.to_str()?)
                                .context("Unable to deserialize response")?;

                        let mut payload: Option<InteractionResponse> = None;

                        // TODO(kijv) classes?
                        // TODO(kijv) make these into functions so Value::Array has no duplicated
                        // code ? string = pure text message
                        // ? array = mix of any of the types with meaning (except array)
                        match initial_return {
                            JsonValue::String(text) => {
                                payload = InteractionResponse {
                                    r#type: InteractionCallbackType::ChannelMessageWithSource,
                                    data: Some(InteractionCallbackData::Messages(
                                        InteractionCallbackMessagesData {
                                            tts: None,
                                            content: Some(text.into()),
                                            embeds: None,
                                            allowed_mentions: None,
                                            flags: None,
                                            components: None,
                                            attachments: None,
                                        },
                                    )),
                                }
                                .into();
                            }
                            JsonValue::Object(map) => {
                                let value = JsonValue::Object(map);

                                // Lilybird embeds in a format that the Discord API
                                // already accepts, so we just send this object to
                                // the response
                                if let Ok(embed) = serde_json::from_value::<lilybird::Embed>(value)
                                {
                                    // Although most values are the same, the accepted value relies
                                    // on the Turbo engine for
                                    // caching abilities
                                    let acceptable_embed = Embed {
                                        title: embed.title.map(|str| str.into()),
                                        r#type: embed.r#type.map(|r#type| match r#type {
                                            lilybird::EmbedType::Rich => EmbedType::Rich,
                                            lilybird::EmbedType::Image => EmbedType::Image,
                                            lilybird::EmbedType::Video => EmbedType::Rich,
                                            lilybird::EmbedType::Gif => EmbedType::Gifv,
                                            lilybird::EmbedType::Article => EmbedType::Article,
                                            lilybird::EmbedType::Link => EmbedType::Link,
                                        }),
                                        description: embed.description.map(|str| str.into()),
                                        url: embed.url.map(|str| str.into()),
                                        timestamp: embed.timestamp.map(|str| str.into()),
                                        color: embed.color,
                                        footer: embed
                                            .footer
                                            .map(|footer| {
                                                Some(EmbedFooter {
                                                    // TODO(kijv) use a helper function to
                                                    // automatically
                                                    // transform certain fields
                                                    text: footer.text.into(),
                                                    icon_url: footer.icon_url.map(|str| str.into()),
                                                    proxy_icon_url: footer
                                                        .proxy_icon_url
                                                        .map(|str| str.into()),
                                                })
                                            })
                                            .unwrap_or_default(),
                                        image: embed
                                            .image
                                            .map(|image| {
                                                Some(EmbedImage {
                                                    url: image.url.into(),
                                                    proxy_url: image
                                                        .proxy_url
                                                        .map(|str| str.into()),
                                                    height: image.height,
                                                    width: image.width,
                                                })
                                            })
                                            .unwrap_or_default(),
                                        thumbnail: embed
                                            .thumbnail
                                            .map(|thumbnail| {
                                                Some(EmbedThumbnail {
                                                    url: thumbnail.url.into(),
                                                    proxy_url: thumbnail
                                                        .proxy_url
                                                        .map(|str| str.into()),
                                                    height: thumbnail.height,
                                                    width: thumbnail.width,
                                                })
                                            })
                                            .unwrap_or_default(),
                                        video: embed
                                            .video
                                            .map(|video| {
                                                Some(EmbedVideo {
                                                    // Discord API technically doesn't require the
                                                    // url
                                                    // (according to the docs)
                                                    url: Some(video.url.into()),
                                                    proxy_url: video
                                                        .proxy_url
                                                        .map(|str| str.into()),
                                                    height: video.height,
                                                    width: video.width,
                                                })
                                            })
                                            .unwrap_or_default(),
                                        provider: embed
                                            .provider
                                            .map(|provider| {
                                                Some(EmbedProvider {
                                                    name: provider.name.map(|str| str.into()),
                                                    url: provider.url.map(|str| str.into()),
                                                })
                                            })
                                            .unwrap_or_default(),
                                        author: embed
                                            .author
                                            .map(|author| {
                                                Some(EmbedAuthor {
                                                    name: author.name.into(),
                                                    url: author.url.map(|str| str.into()),
                                                    icon_url: author.icon_url.map(|str| str.into()),
                                                    proxy_icon_url: author
                                                        .proxy_icon_url
                                                        .map(|str| str.into()),
                                                })
                                            })
                                            .unwrap_or_default(),
                                        fields: embed
                                            .fields
                                            .map(|fields| {
                                                Some(
                                                    fields
                                                        .iter()
                                                        .map(|field| EmbedField {
                                                            name: field.clone().name.into(),
                                                            value: field.clone().value.into(),
                                                            inline: field.inline,
                                                        })
                                                        .collect(),
                                                )
                                            })
                                            .unwrap_or_default(),
                                    };
                                    payload = InteractionResponse {
                                        r#type: InteractionCallbackType::ChannelMessageWithSource,
                                        data: Some(InteractionCallbackData::Messages(
                                            InteractionCallbackMessagesData {
                                                tts: None,
                                                content: None,
                                                embeds: Some(vec![acceptable_embed]),
                                                allowed_mentions: None,
                                                flags: None,
                                                components: None,
                                                attachments: None,
                                            },
                                        )),
                                    }
                                    .into();
                                }
                            }
                            JsonValue::Array(_) => {
                                RuntimeIssue {
                                    path: entry.ident().path(),
                                    severity: Some(IssueSeverity::Error.cell()),
                                    title: "Failed to parse command response".into(),
                                    description: Some("Parsing arrays is not supported yet".into()),
                                }
                                .cell()
                                .emit();
                            }
                            JsonValue::Null => {}
                            json_value => {
                                let r#typeof = match json_value {
                                    JsonValue::Bool(_) => "boolean",
                                    JsonValue::Number(_) => "number",
                                    _ => unreachable!(),
                                };

                                RuntimeIssue {
                                    path: entry.ident().path(),
                                    severity: Some(IssueSeverity::Error.cell()),
                                    title: "Failed to parse command response".into(),
                                    description: Some(
                                        format!(
                                            "Return value of type {} is not supported",
                                            r#typeof
                                        )
                                        .into(),
                                    ),
                                }
                                .cell()
                                .emit();
                            }
                        };

                        if let Some(payload) = payload {
                            let interaction_id: InteractionId =
                                data["id"].as_str().unwrap().parse().unwrap();

                            // should be passed to edit_interaction_response
                            let error_interaction_response =
                                discord_api::rest::interactions::create_interaction_response(
                                    interaction_id.cell(),
                                    Vc::cell(data["token"].as_str().unwrap().to_string().into()),
                                    payload.cell(),
                                    Some(entry.ident().path()),
                                );

                            return handle_issues(
                                error_interaction_response,
                                issue_reporter,
                                IssueSeverity::Fatal.cell(),
                                None,
                                Some("create interaction response"),
                            )
                            .await;
                        }

                        Ok(())
                    },
                ));
                ctx.ongoing_side_effects
                    .lock()
                    .await
                    .push_back(Arc::new(Mutex::new(Some(join_handle))));
            }

            Ok(Default::default())
        }
        _ => {
            let resolved_source = source.resolve_strongly_consistent().await?;

            // User-provided event
            if let Some(event_entry) = get_event_entry(resolved_source, event_name.clone())
                .await?
                .as_ref()
            {
                let project_path = resolved_source.clone().await?.project_path;
                let chunking_context = resolved_source
                    .clone()
                    .await?
                    .executor
                    .await?
                    .chunking_context;
                let asset_context = resolved_source.clone().await?.executor.await?.asset_context;
                let env = resolved_source.clone().await?.executor.await?.env;

                let initial_val = evaluate(
                    *event_entry,
                    project_path,
                    env,
                    FileSource::new(project_path).ident(),
                    asset_context,
                    Vc::upcast(chunking_context),
                    None,
                    vec![],
                    resolved_source
                        .get_events_dir()
                        .routes_changed(ctx.await?.config),
                    ctx.await?.debug,
                );

                let SingleValue::Single(_) = initial_val.await?.try_into_single().await? else {
                    // An error happened, which has already been converted into an issue.
                    handle_issues(
                        initial_val,
                        issue_reporter,
                        IssueSeverity::Fatal.cell(),
                        None,
                        Some("parse evaluate response"),
                    )
                    .await?;

                    return Ok(Default::default());
                };
            }

            Ok(Default::default())
        }
    }
}

#[turbo_tasks::function]
pub async fn get_event_entry(
    resolved_source: Vc<ContentSourceData>,
    event_name: RcStr,
) -> Result<Vc<OptionModule>> {
    let events = resolved_source.get_events().await?;
    let events = events
        .iter()
        .map(|v| (v.name.clone(), v.file_path.clone()))
        .collect::<BTreeMap<_, _>>();
    let entries = resolved_source.clone().get_entries();

    if let Some(file_path) = events.get(&event_name) {
        Ok(Vc::cell(*entries.get_entry(file_path.to_string()).await?))
    } else {
        Ok(Vc::cell(None))
    }
}
