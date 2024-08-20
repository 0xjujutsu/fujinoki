use std::collections::HashMap;

use anyhow::Result;
use reqwest::header;
use serde_json::Value as JsonValue;
use turbopack_binding::{
    turbo::{
        tasks as turbo_tasks,
        tasks::{RcStr, Vc},
        tasks_fs::FileSystemPath,
    },
    turbopack::core::issue::{IssueExt, IssueSeverity, StyledString},
};

use super::HTTP;
use crate::{
    application::command::ApplicationCommand,
    id::{ApplicationId, CommandId},
    issue::DiscordApiIssue,
    rest::http::fetch_error_to_string,
    Routes,
};

#[turbo_tasks::function]
pub async fn create_global_application_command(
    application_id: ApplicationId,
    application_command: Vc<JsonValue>,
    token: Vc<RcStr>,
    file_path: Option<Vc<FileSystemPath>>,
) -> Result<Vc<()>> {
    let issue_title = "Create global application command".to_string().into();
    let response = *HTTP
        .post(
            Routes::application_commands(application_id),
            application_command,
            Vc::cell(
                HashMap::from_iter(vec![(
                    header::AUTHORIZATION.to_string().into(),
                    format!("Bot {}", token.await?).into(),
                )])
                .into(),
            ),
        )
        .await?;

    match response {
        Ok(_) => Ok(Default::default()),
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

pub async fn get_global_application_command(
    application_id: ApplicationId,
    command_id: CommandId,
    token: RcStr,
    file_path: Option<Vc<FileSystemPath>>,
) -> Result<Vc<Option<Vc<ApplicationCommand>>>> {
    let issue_title = "Get global application command".to_string().into();
    let response = *HTTP
        .get(
            Routes::application_command(application_id, command_id),
            Vc::cell(
                HashMap::from_iter(vec![(
                    header::AUTHORIZATION.to_string().into(),
                    format!("Bot {}", token).into(),
                )])
                .into(),
            ),
            Vc::cell(None),
        )
        .await?;

    match response {
        Ok(response) => {
            let response = response.await?;

            if response.status.eq(&reqwest::StatusCode::OK) {
                let body = response.body.await?;
                let text = serde_json::to_string(&body.0)?;

                return Ok(match serde_json::from_str::<ApplicationCommand>(&text) {
                    Ok(application_command) => Vc::cell(Some(application_command.cell())),
                    Err(err) => {
                        DiscordApiIssue {
                            severity: IssueSeverity::Error.cell(),
                            file_path,
                            title: Some(issue_title),
                            message: StyledString::Text(
                                format!("Failed to transform response into JSON: {err}",).into(),
                            )
                            .cell(),
                        }
                        .cell()
                        .emit();

                        Vc::cell(None)
                    }
                });
            }

            Ok(Vc::cell(None))
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
pub async fn edit_global_application_command(
    application_id: ApplicationId,
    command_id: CommandId,
    optional_application_command: Vc<JsonValue>,
    token: Vc<RcStr>,
    file_path: Option<Vc<FileSystemPath>>,
) -> Result<Vc<()>> {
    let issue_title = "Edit global application command".to_string().into();
    let response = *HTTP
        .patch(
            Routes::application_command(application_id, command_id),
            optional_application_command,
            Vc::cell(
                HashMap::from_iter(vec![(
                    header::AUTHORIZATION.to_string().into(),
                    format!("Bot {}", token.await?).into(),
                )])
                .into(),
            ),
        )
        .await?;

    match response {
        Ok(_) => Ok(Default::default()),
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
pub async fn delete_global_application_command(
    application_id: ApplicationId,
    command_id: CommandId,
    token: Vc<RcStr>,
    file_path: Option<Vc<FileSystemPath>>,
) -> Result<Vc<()>> {
    let issue_title = "Delete global application command".to_string().into();
    let response = *HTTP
        .delete(
            Routes::application_command(application_id, command_id),
            Vc::cell(
                HashMap::from_iter(vec![(
                    header::AUTHORIZATION.to_string().into(),
                    format!("Bot {}", token.await?).into(),
                )])
                .into(),
            ),
            Vc::cell(None),
        )
        .await?;

    match response {
        Ok(_) => Ok(Default::default()),
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
