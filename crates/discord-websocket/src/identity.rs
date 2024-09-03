// TODO better generics for data that is passed around (ctx.config)
use std::sync::Arc;

use anyhow::{anyhow, Result};
use discord_api::gateway::{OpCode, Payload};
use futures::SinkExt;
use serde_json::json;
use tokio_tungstenite::tungstenite::Message;
use turbopack_binding::{
    turbo::tasks::{run_once_with_reason, TurboTasksApi, Vc},
    turbopack::core::issue::{handle_issues, IssueReporter, IssueSeverity},
};

use crate::{context::WebsocketContext, invalidation::WebsocketMessage};

pub async fn identify(
    tt: Arc<dyn TurboTasksApi>,
    get_issue_reporter: Arc<dyn Fn() -> Vc<Box<dyn IssueReporter>> + Send + Sync>,
    ctx: WebsocketContext,
) -> Result<()> {
    let reason = WebsocketMessage {
        opcode: OpCode::Identify,
        event: None,
        hide: false,
    };

    run_once_with_reason(tt.clone(), reason, async move {
        let issue_reporter = get_issue_reporter();
        let mut write = ctx
            .api
            .write
            .try_lock()
            .expect("failed to lock `write` stream");

        // TODO(kijv) instead of taking fujinoki config, take in client config (intents, token, etc.)
        let token = ctx.config.client().token();

        handle_issues(
            token,
            issue_reporter,
            IssueSeverity::Fatal.cell(),
            None,
            Some("get config token"),
        )
        .await?;

        let intents = ctx.config.client().intents();

        let payload = Payload {
            op: OpCode::Identify,
            d: Some(json!({
                "token": token.await?,
                "intents": intents.await?,
                "properties": {
                    "os": std::env::consts::OS,
                    "browser": "fujinoki",
                    "device": "fujinoki"
                }
            })),
            s: None,
            t: None,
        };

        write
            .send(Message::Text(serde_json::to_string(&payload)?))
            .await
            .map_err(|err| anyhow!(err))
    })
    .await
}
