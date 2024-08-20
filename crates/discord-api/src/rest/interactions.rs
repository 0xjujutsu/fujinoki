use anyhow::Result;
use turbopack_binding::{
    turbo::{
        tasks as turbo_tasks,
        tasks::{RcStr, Vc},
        tasks_fs::FileSystemPath,
    },
    turbopack::core::issue::{IssueExt, IssueSeverity, StyledString},
};

use super::{http::fetch_error_to_string, routes::MessageIdOrOriginal, HTTP};
use crate::{
    channel::{
        embed::{Embed, EmbedField},
        message::Message,
    },
    id::{ApplicationId, InteractionId},
    interactions::{
        InteractionCallbackData, InteractionCallbackMessagesData, InteractionCallbackType,
        InteractionResponse,
    },
    issue::DiscordApiIssue,
    utils::title::title_case,
    Routes,
};

#[turbo_tasks::function]
pub async fn create_interaction_response(
    interaction_id: Vc<InteractionId>,
    interaction_token: Vc<RcStr>,
    interaction_response: Vc<InteractionResponse>,
    file_path: Option<Vc<FileSystemPath>>,
) -> Result<Vc<()>> {
    let issue_title = "Create interaction response".to_string().into();
    let request_body = serde_json::to_value(interaction_response.await?);

    // TODO make generic function so we don't have to keep repeating this
    if let Some(err) = request_body.as_ref().err() {
        DiscordApiIssue {
            severity: IssueSeverity::Error.cell(),
            file_path,
            title: Some(issue_title),
            message: StyledString::Text(format!("Failed to serialize request body: {err}").into())
                .cell(),
        }
        .cell()
        .emit();

        return Ok(Vc::cell(()));
    }

    let response = *HTTP
        .post(
            Routes::interaction_callback(
                interaction_id.await?.clone_value().into(),
                interaction_token.await?.clone_value().to_string().into(),
            ),
            Vc::cell(request_body.unwrap()),
            Vc::cell(None),
        )
        .await?;

    match response {
        Ok(response) => {
            let response = response.await?;

            if response.status.eq(&reqwest::StatusCode::NO_CONTENT) {
                Ok(Vc::cell(()))
            } else {
                let body = response.body.await?;
                let text = serde_json::to_string(&body.0)?;

                match serde_json::from_str::<serde_json::Value>(&text) {
                    Ok(json) => {
                        let mut fields: Vec<EmbedField> = vec![];
                        if let Some(errors) = json["errors"].as_object() {
                            for (key, json) in errors {
                                for (field, _) in json.as_object().unwrap() {
                                    let field_errors = errors[key][field]["_errors"].as_array();

                                    if let Some(field_errors) = field_errors {
                                        for error in field_errors {
                                            let message = error["message"].as_str().unwrap();
                                            DiscordApiIssue {
                                                severity: IssueSeverity::Error.cell(),
                                                file_path,
                                                title: Some(title_case(field, None).into()),
                                                message: StyledString::Text(
                                                    message.to_string().into(),
                                                )
                                                .cell(),
                                            }
                                            .cell()
                                            .emit();
                                        }
                                    }
                                }
                            }
                        } else {
                            let message = json["message"].as_str().unwrap();
                            let code = json["code"].as_u64().unwrap();

                            DiscordApiIssue {
                                severity: IssueSeverity::Error.cell(),
                                file_path,
                                title: Some(issue_title),
                                message: StyledString::Text(
                                    format!("{}: {}", code, message).into(),
                                )
                                .cell(),
                            }
                            .cell()
                            .emit();

                            fields.push(EmbedField {
                                name: message.to_string().into(),
                                value: "abc".to_string().into(),
                                inline: None,
                            })
                        }

                        if fields.len() > 0 {
                            // TODO finish this
                            let _payload = InteractionResponse {
                                r#type: InteractionCallbackType::ChannelMessageWithSource,
                                data: Some(InteractionCallbackData::Messages(
                                    InteractionCallbackMessagesData {
                                        tts: None,
                                        content: None,
                                        embeds: Some(vec![Embed {
                                            title: Some("Errors".to_string().into()),
                                            r#type: None,
                                            description: None,
                                            url: None,
                                            timestamp: None,
                                            color: None,
                                            footer: None,
                                            image: None,
                                            thumbnail: None,
                                            video: None,
                                            provider: None,
                                            author: None,
                                            fields: Some(fields),
                                        }]),
                                        allowed_mentions: None,
                                        flags: None,
                                        components: None,
                                        attachments: None,
                                    },
                                )),
                            };
                            // edit_interaction_response(
                            //     application_id,
                            //     interaction_token,
                            //     payload.cell(),
                            //     file_path,
                            // )
                            // .await?;
                        }
                    }
                    Err(_) => {
                        if text.trim().len() > 0 {
                            DiscordApiIssue {
                                severity: IssueSeverity::Error.cell(),
                                file_path,
                                title: Some(issue_title),
                                message: StyledString::Text(text.into()).cell(),
                            }
                            .cell()
                            .emit();
                        }
                    }
                }

                Ok(Vc::cell(()))
            }
        }
        Err(err) => {
            let err = fetch_error_to_string(err).await?;

            DiscordApiIssue {
                severity: IssueSeverity::Error.cell(),
                file_path,
                title: Some(issue_title),
                message: StyledString::Text(format!("Received error: {err}").into()).cell(),
            }
            .cell()
            .emit();

            return Ok(Default::default());
        }
    }
}

#[turbo_tasks::function]
pub async fn get_interaction_response(
    application_id: ApplicationId,
    interaction_token: RcStr,
    file_path: Option<Vc<FileSystemPath>>,
) -> Result<Vc<Option<Vc<Message>>>> {
    let issue_title = "Get interaction response".to_string().into();
    let response = *HTTP
        .get(
            Routes::webhook_message(
                application_id.into(),
                interaction_token.to_string().into(),
                MessageIdOrOriginal::Original,
            ),
            Vc::cell(None),
            Vc::cell(None),
        )
        .await?;

    match response {
        Ok(response) => {
            let response = response.await?;

            if response.status.eq(&reqwest::StatusCode::OK) {
                let body = response.body.await?;
                let text = serde_json::to_string(&body.0)?;

                match serde_json::from_str::<Message>(&text) {
                    Ok(message) => Ok(Vc::cell(Some(message.cell()))),
                    Err(err) => {
                        DiscordApiIssue {
                            severity: IssueSeverity::Error.cell(),
                            file_path,
                            title: Some(issue_title),
                            message: StyledString::Text(
                                format!(
                                    "Failed to transform response into JSON: {}",
                                    err.to_string()
                                )
                                .into(),
                            )
                            .cell(),
                        }
                        .cell()
                        .emit();

                        Ok(Default::default())
                    }
                }
            } else {
                let body = response.body.await?;
                let text = serde_json::to_string(&body.0)?;

                match serde_json::from_str::<serde_json::Value>(&text) {
                    Ok(json) => {
                        let fields: Vec<EmbedField> = vec![];
                        if let Some(errors) = json["errors"].as_object() {
                            for (key, json) in errors {
                                for (field, _) in json.as_object().unwrap() {
                                    let field_errors = errors[key][field]["_errors"].as_array();

                                    if let Some(field_errors) = field_errors {
                                        for error in field_errors {
                                            let message = error["message"].as_str().unwrap();
                                            DiscordApiIssue {
                                                severity: IssueSeverity::Error.cell(),
                                                file_path,
                                                title: Some(title_case(field, None).into()),
                                                message: StyledString::Text(
                                                    message.to_string().into(),
                                                )
                                                .cell(),
                                            }
                                            .cell()
                                            .emit();
                                        }
                                    }
                                }
                            }
                        } else {
                            let message = json["message"].as_str().unwrap();
                            let code = json["code"].as_u64().unwrap();

                            DiscordApiIssue {
                                severity: IssueSeverity::Error.cell(),
                                file_path,
                                title: Some(issue_title),
                                message: StyledString::Text(
                                    format!("{}: {}", code, message).into(),
                                )
                                .cell(),
                            }
                            .cell()
                            .emit();
                        }

                        if fields.len() > 0 {
                            let payload: InteractionResponse = InteractionResponse {
                                r#type: InteractionCallbackType::ChannelMessageWithSource,
                                data: Some(InteractionCallbackData::Messages(
                                    InteractionCallbackMessagesData {
                                        tts: None,
                                        content: None,
                                        embeds: Some(vec![Embed {
                                            title: Some("Errors".to_string().into()),
                                            r#type: None,
                                            description: None,
                                            url: None,
                                            timestamp: None,
                                            color: None,
                                            footer: None,
                                            image: None,
                                            thumbnail: None,
                                            video: None,
                                            provider: None,
                                            author: None,
                                            fields: Some(fields),
                                        }]),
                                        allowed_mentions: None,
                                        flags: None,
                                        components: None,
                                        attachments: None,
                                    },
                                )),
                            };
                            edit_interaction_response(
                                application_id,
                                interaction_token,
                                payload.cell(),
                                file_path,
                            )
                            .await?;
                        }
                    }
                    Err(_) => {
                        if text.trim().len() > 0 {
                            DiscordApiIssue {
                                severity: IssueSeverity::Error.cell(),
                                file_path,
                                title: Some(issue_title),
                                message: StyledString::Text(text.into()).cell(),
                            }
                            .cell()
                            .emit();
                        }
                    }
                }

                Ok(Default::default())
            }
        }
        Err(err) => {
            let err = fetch_error_to_string(err).await?;

            DiscordApiIssue {
                severity: IssueSeverity::Error.cell(),
                file_path,
                title: Some(issue_title),
                message: StyledString::Text(format!("Received error: {err}").into()).cell(),
            }
            .cell()
            .emit();

            Ok(Default::default())
        }
    }
}

#[turbo_tasks::function]
pub async fn edit_interaction_response(
    application_id: ApplicationId,
    interaction_token: RcStr,
    interaction_response: Vc<InteractionResponse>,
    file_path: Option<Vc<FileSystemPath>>,
) -> Result<Vc<()>> {
    let issue_title = "Edit interaction response".to_string().into();
    let request_body = serde_json::to_value(interaction_response.await?);

    if let Some(err) = request_body.as_ref().err() {
        DiscordApiIssue {
            severity: IssueSeverity::Error.cell(),
            file_path,
            title: Some(issue_title),
            message: StyledString::Text(format!("Failed to serialize request body: {err}").into())
                .cell(),
        }
        .cell()
        .emit();

        return Ok(Default::default());
    }

    let response = *HTTP
        .patch(
            Routes::webhook_message(
                application_id.into(),
                interaction_token.to_string().into(),
                MessageIdOrOriginal::Original,
            ),
            Vc::cell(request_body.unwrap()),
            Vc::cell(None),
        )
        .await?;

    match response {
        Ok(response) => {
            let response = response.await?;

            if response.status.eq(&reqwest::StatusCode::NO_CONTENT) {
                Ok(Default::default())
            } else {
                let body = response.body.await?;
                let text = serde_json::to_string(&body.0)?;

                match serde_json::from_str::<serde_json::Value>(&text) {
                    Ok(json) => {
                        let fields: Vec<EmbedField> = vec![];
                        if let Some(errors) = json["errors"].as_object() {
                            for (key, json) in errors {
                                for (field, _) in json.as_object().unwrap() {
                                    let field_errors = errors[key][field]["_errors"].as_array();

                                    if let Some(field_errors) = field_errors {
                                        for error in field_errors {
                                            let message = error["message"].as_str().unwrap();
                                            DiscordApiIssue {
                                                severity: IssueSeverity::Error.cell(),
                                                file_path,
                                                title: Some(title_case(field, None).into()),
                                                message: StyledString::Text(
                                                    message.to_string().into(),
                                                )
                                                .cell(),
                                            }
                                            .cell()
                                            .emit();
                                        }
                                    }
                                }
                            }
                        } else {
                            let message = json["message"].as_str().unwrap();
                            let code = json["code"].as_u64().unwrap();

                            DiscordApiIssue {
                                severity: IssueSeverity::Error.cell(),
                                file_path,
                                title: Some(issue_title),
                                message: StyledString::Text(
                                    format!("{}: {}", code, message).into(),
                                )
                                .cell(),
                            }
                            .cell()
                            .emit();
                        }

                        if fields.len() > 0 {
                            // TODO add info
                            let _payload: InteractionResponse = InteractionResponse {
                                r#type: InteractionCallbackType::ChannelMessageWithSource,
                                data: Some(InteractionCallbackData::Messages(
                                    InteractionCallbackMessagesData {
                                        tts: None,
                                        content: None,
                                        embeds: Some(vec![Embed {
                                            title: Some("Errors".to_string().into()),
                                            r#type: None,
                                            description: None,
                                            url: None,
                                            timestamp: None,
                                            color: None,
                                            footer: None,
                                            image: None,
                                            thumbnail: None,
                                            video: None,
                                            provider: None,
                                            author: None,
                                            fields: None,
                                        }]),
                                        allowed_mentions: None,
                                        flags: None,
                                        components: None,
                                        attachments: None,
                                    },
                                )),
                            };
                        }
                    }
                    Err(_) => {
                        if text.trim().len() > 0 {
                            DiscordApiIssue {
                                severity: IssueSeverity::Error.cell(),
                                file_path,
                                title: Some(issue_title),
                                message: StyledString::Text(text.into()).cell(),
                            }
                            .cell()
                            .emit();
                        }
                    }
                }

                Ok(Default::default())
            }
        }
        Err(err) => {
            let err = fetch_error_to_string(err).await?;
            DiscordApiIssue {
                severity: IssueSeverity::Error.cell(),
                file_path,
                title: Some(issue_title),
                message: StyledString::Text(format!("Received error: {err}").into()).cell(),
            }
            .cell()
            .emit();

            return Ok(Default::default());
        }
    }
}
