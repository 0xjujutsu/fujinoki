use std::{sync::Arc, time::Duration};

use anyhow::{anyhow, Result};
use discord_api::gateway::{opcode::OpCode, payload::Payload};
use discord_websocket::invalidation::WebsocketMessage;
use futures::{SinkExt, StreamExt};
use serde_json::json;
use tokio_tungstenite::tungstenite::{protocol::CloseFrame, Message};
use tracing::{event, Level, Span};
use turbopack_binding::{
    features::auto_hash_map::AutoSet,
    turbo::tasks::{run_once, run_once_with_reason, CollectiblesSource, TurboTasksApi, Vc},
    turbopack::core::issue::{handle_issues, IssueExt, IssueReporter, IssueSeverity},
};
use url::Url;

use super::WebsocketContext;
use crate::{
    connect_to_gateway,
    issue::WebsocketIssue,
    source::ContentSourceSideEffect,
    SourceProvider,
};

pub struct WebsocketEvents {
    tt: Arc<dyn TurboTasksApi>,
    ctx: WebsocketContext,
    get_issue_reporter: Arc<dyn Fn() -> Vc<Box<dyn IssueReporter>> + Send + Sync>,
}

impl WebsocketEvents {
    pub fn new(
        tt: Arc<dyn TurboTasksApi>,
        ctx: WebsocketContext,
        get_issue_reporter: Arc<dyn Fn() -> Vc<Box<dyn IssueReporter>> + Send + Sync>,
    ) -> Self {
        Self {
            tt,
            ctx,
            get_issue_reporter,
        }
    }

    // use something like concurrent-queue to make sure we don't hit any rate
    // limits
    pub async fn text(
        &self,
        source_provider: impl SourceProvider + Sync,
        text: String,
    ) -> Result<()> {
        let ctx = self.ctx.clone();

        if !text.starts_with("{") && !text.ends_with("}") {
            dbg!(text);
            return Ok(());
        }

        let json = serde_json::from_str::<Payload>(&*text).unwrap();

        {
            let mut sequence = ctx.sequence.try_lock().expect("failed to lock `sequence`");
            *sequence = json.s;
        }

        let empty_obj = json!({});
        let tt = self.tt.clone();
        let get_issue_reporter = self.get_issue_reporter.clone();
        let source_provider = source_provider.clone();
        let json_cloned = json.clone();
        let data = json_cloned.d.clone();

        event!(parent: Span::current(), Level::DEBUG, "gateway data received");

        let reason = WebsocketMessage {
            opcode: json_cloned.op,
            event: json_cloned.t.map(|t| t.to_string()),
            hide: json_cloned.op == OpCode::Heartbeat || json_cloned.op == OpCode::HeartbeatAck,
        };

        run_once_with_reason(tt.clone(), reason, async move {
            let data = data.clone();

            // match json.op {
            //     OpCode::Dispatch => {
            //         if json.t.is_some() {
            //             // TODO
            //             // let result = discord_websocket::receive::dispatch(
            //             //     json.cell(),
            //             //     source_provider.get_source(),
            //             //     get_issue_reporter(),
            //             //     ctx.cell(),
            //             // );

            //             // TODO make actual side effects
            //             // let _se: AutoSet<Vc<Box<dyn ContentSourceSideEffect>>> =
            //             //     result.peek_collectibles();

            //             // handle_issues(
            //             //     result,
            //             //     get_issue_reporter(),
            //             //     IssueSeverity::Fatal.cell(),
            //             //     None,
            //             //     Some("DISPATCH"),
            //             // )
            //             // .await?;
            //         }
            //     }
            //     // TODO redo (match djs?)
            //     OpCode::Reconnect => {
            //         let resume_gateway_url = ctx
            //             .resume_gateway_url
            //             .try_lock()
            //             .expect("failed to lock `resume_gateway_url`");
            //         let session_id = ctx
            //             .session_id
            //             .try_lock()
            //             .expect("failed to lock `session_id`");
            //         let sequence = ctx.sequence.try_lock().expect("failed to lock `sequence`");

            //         let payload = json!({
            //             "op": OpCode::Resume,
            //             "d": {
            //                 "token": ctx.config.client().token().await?,
            //                 "session_id": *session_id,
            //                 "seq": *sequence
            //             }
            //         });

            //         let (new_write, new_read) =
            //             connect_to_gateway(resume_gateway_url.clone(), discord_api::VERSION)
            //                 .await?
            //                 .split();

            //         let mut write = ctx
            //             .api
            //             .write
            //             .try_lock()
            //             .expect("failed to lock `write` stream");
            //         let mut read = ctx
            //             .api
            //             .read
            //             .try_lock()
            //             .expect("failed to lock `read` stream");
            //         *write = new_write;
            //         *read = new_read;

            //         write.send(Message::Text(payload.to_string())).await?;
            //     }
            //     // TODO redo (match djs?)
            //     OpCode::InvalidSession => {
            //         let should_reconnect = raw_data
            //             .unwrap_or(serde_json::to_value(false).unwrap())
            //             .as_bool()
            //             .unwrap_or(false);

            //         if should_reconnect {
            //             let resume_gateway_url = ctx
            //                 .resume_gateway_url
            //                 .try_lock()
            //                 .expect("failed to lock `resume_gateway_url`");
            //             let session_id = ctx
            //                 .session_id
            //                 .try_lock()
            //                 .expect("failed to lock `session_id`");
            //             let sequence = ctx.sequence.try_lock().expect("failed to lock `sequence`");

            //             let payload = json!({
            //                 "op": OpCode::Resume,
            //                 "d": {
            //                     "token": ctx.config.client().token().await?,
            //                     "session_id": *session_id,
            //                     "seq": *sequence
            //                 }
            //             });

            //             let url = Url::parse(&*resume_gateway_url.clone().unwrap()).unwrap();
            //             let (new_write, new_read) =
            //                 connect_to_gateway(Some(url.to_string()), discord_api::VERSION)
            //                     .await?
            //                     .split();

            //             let mut write = ctx
            //                 .api
            //                 .write
            //                 .try_lock()
            //                 .expect("failed to lock `write` stream");
            //             let mut read = ctx
            //                 .api
            //                 .read
            //                 .try_lock()
            //                 .expect("failed to lock `read` stream");
            //             *write = new_write;
            //             *read = new_read;

            //             write.send(Message::Text(payload.to_string())).await?;
            //         }
            //     }
            //     OpCode::Hello => {
            //         let heartbeat_interval = ctx.heartbeat_interval.clone();
            //         let mut heartbeat_interval = heartbeat_interval
            //             .try_lock()
            //             .expect("failed to lock `heartbeat_interval`");

            //         *heartbeat_interval = data["heartbeat_interval"]
            //             .as_u64()
            //             .map(Duration::from_millis);

            //         // so we can re-lock in the heartbeat function
            //         drop(heartbeat_interval);

            //         discord::heartbeat(ctx.clone(), true).await?;
            //     }
            //     OpCode::Heartbeat => {
            //         discord_websocket::send::heartbeat(ctx.clone(), true).await?;
            //     }
            //     OpCode::HeartbeatAck => {
            //         let heartbeat_ack = ctx.heartbeat_ack.clone();
            //         let mut heartbeat_ack = heartbeat_ack
            //             .try_lock()
            //             .expect("failed to lock `heartbeat_ack`");

            //         *heartbeat_ack = true;
            //     }
            //     _ => {
            //         dbg!(data);
            //     }
            // }

            Ok(())
        })
        .await
    }

    pub async fn close(
        &self,
        source_provider: impl SourceProvider + Sync,
        message: Option<CloseFrame<'static>>,
    ) -> Result<()> {
        match message {
            Some(message) => {
                let reconnect_codes = vec![4000, 4001, 4002, 4003, 4005, 4007, 4008, 4009];

                if reconnect_codes.contains(&message.code.into()) {
                    let resume_gateway_url = self
                        .ctx
                        .resume_gateway_url
                        .try_lock()
                        .expect("failed to lock `resume_gateway_url`");
                    let session_id = self
                        .ctx
                        .session_id
                        .try_lock()
                        .expect("failed to lock `session_id`");
                    let sequence = self
                        .ctx
                        .sequence
                        .try_lock()
                        .expect("failed to lock `sequence`");

                    let payload = json!({
                        "op": OpCode::Resume,
                        "d": {
                            "token": self.ctx.config.client().token().await?,
                            "session_id": *session_id,
                            "seq": *sequence
                        }
                    });

                    let url = Url::parse(&*resume_gateway_url.clone().unwrap()).unwrap();
                    let (new_write, new_read) =
                        connect_to_gateway(Some(url.to_string()), discord_api::VERSION)
                            .await?
                            .split();

                    let mut write = self
                        .ctx
                        .api
                        .write
                        .try_lock()
                        .expect("failed to lock `write` stream");
                    let mut read = self
                        .ctx
                        .api
                        .read
                        .try_lock()
                        .expect("failed to lock `read` stream");
                    *write = new_write;
                    *read = new_read;

                    write
                        .send(Message::Text(payload.to_string()))
                        .await
                        .map_err(|err| anyhow!("{err}"))
                } else {
                    let reason = match message.reason.to_string().is_empty() {
                        true => "Unknown reason".to_string(),
                        false => message.reason.to_string(),
                    };

                    let get_issue_reporter = self.get_issue_reporter.clone();
                    let source_provider = source_provider.clone();

                    run_once(self.tt.clone(), async move {
                        let issue_reporter = get_issue_reporter();
                        let source_provider = source_provider.clone();
                        let source = source_provider.get_source();
                        let resolved_source = source.resolve_strongly_consistent().await?;

                        handle_issues(
                            source,
                            issue_reporter,
                            IssueSeverity::Fatal.cell(),
                            None,
                            None,
                        )
                        .await?;

                        let issue = WebsocketIssue {
                            path: resolved_source.await?.project_path,
                            title: "Connection closed".into(),
                            description: Some(format!("{reason} ({})", message.code).into()),
                        }
                        .cell();
                        issue.emit();

                        handle_issues(
                            issue,
                            issue_reporter,
                            IssueSeverity::Fatal.cell(),
                            None,
                            None,
                        )
                        .await?;

                        Ok(())
                    })
                    .await
                }
            }
            None => {
                let get_issue_reporter = self.get_issue_reporter.clone();
                let source_provider = source_provider.clone();

                run_once(self.tt.clone(), async move {
                    let issue_reporter = get_issue_reporter();
                    let source_provider = source_provider.clone();
                    let source = source_provider.get_source();
                    let resolved_source = source.resolve_strongly_consistent().await?;

                    handle_issues(
                        source,
                        issue_reporter,
                        IssueSeverity::Fatal.cell(),
                        None,
                        None,
                    )
                    .await?;

                    let issue = WebsocketIssue {
                        path: resolved_source.await?.project_path,
                        title: "Connection closed".into(),
                        description: None,
                    }
                    .cell();
                    issue.emit();

                    handle_issues(
                        issue,
                        issue_reporter,
                        IssueSeverity::Fatal.cell(),
                        None,
                        None,
                    )
                    .await?;

                    Ok(())
                })
                .await
            }
        }
    }
}
