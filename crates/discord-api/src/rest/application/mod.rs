// TODO use struct with traits so we don't have to repeat token
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

pub mod command;
pub mod commands;

use super::{http::fetch_error_to_string, HTTP};
use crate::{application::Application, issue::DiscordApiIssue, Routes};

#[turbo_tasks::value(transparent)]
pub struct OptionApplication(Option<Vc<Application>>);

#[turbo_tasks::function]
pub async fn get_current_application(
    token: RcStr,
    file_path: Option<Vc<FileSystemPath>>,
) -> Result<Vc<OptionApplication>> {
    let issue_title = "Get current application".to_string().into();
    let response = *HTTP
        .get(
            Routes::current_application(),
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

                return Ok(match serde_json::from_str::<Application>(&text) {
                    Ok(application) => Vc::cell(Some(application.cell())),
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

            Ok(Vc::cell(None))
        }
    }
}
