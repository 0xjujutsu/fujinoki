use std::collections::HashMap;

use anyhow::Result;
use reqwest::header;
use turbopack_binding::{
    turbo::{
        tasks as turbo_tasks,
        tasks::{RcStr, Vc},
        tasks_fs::FileSystemPath,
    },
    turbopack::core::issue::{IssueExt, IssueSeverity, StyledString},
};

use crate::{
    application::command::ApplicationCommand,
    id::ApplicationId,
    issue::DiscordApiIssue,
    rest::{http::fetch_error_to_string, HTTP},
    Routes,
};

#[turbo_tasks::function]
pub async fn get_global_application_commands(
    application_id: ApplicationId,
    with_localizations: bool,
    token: Vc<RcStr>,
    file_path: Option<Vc<FileSystemPath>>,
) -> Result<Vc<Vec<Vc<ApplicationCommand>>>> {
    let issue_title = "Get global application commands".to_string().into();
    let response = *HTTP
        .get(
            Routes::application_commands(application_id),
            Vc::cell(
                HashMap::from_iter(vec![(
                    header::AUTHORIZATION.to_string().into(),
                    format!("Bot {}", token.await?.clone()).into(),
                )])
                .into(),
            ),
            Vc::cell(
                vec![(
                    "with_localizations".into(),
                    with_localizations.to_string().into(),
                )]
                .into(),
            ),
        )
        .await?;

    match response {
        Ok(response) => {
            let response = response.await?;

            if response.status.eq(&reqwest::StatusCode::OK) {
                let body = response.body.await?;
                let text = serde_json::to_string(&body.0)?;

                return Ok(
                    match serde_json::from_str::<Vec<ApplicationCommand>>(&text) {
                        Ok(application_commands) => {
                            let application_commands = application_commands
                                .into_iter()
                                .map(|application_command| application_command.cell())
                                .collect::<Vec<_>>();

                            Vc::cell(application_commands)
                        }
                        Err(err) => {
                            DiscordApiIssue {
                                severity: IssueSeverity::Error.cell(),
                                file_path,
                                title: Some(issue_title),
                                message: StyledString::Text(
                                    format!("Failed to transform response into JSON: {err}",)
                                        .into(),
                                )
                                .cell(),
                            }
                            .cell()
                            .emit();

                            Default::default()
                        }
                    },
                );
            }

            Ok(Vc::cell(vec![]))
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
