use std::sync::Arc;

use anyhow::{anyhow, Result};
use discord_api::gateway::{
    opcode::OpCode,
    payload::{
        send::{ConnectionProperties, IdentifyPayloadData},
        Payload, PayloadData, SendEvents,
    },
};
use futures::SinkExt;
use tokio_tungstenite::tungstenite::Message;
use turbopack_binding::turbo::tasks::{
    run_once_with_reason, TurboTasksApi, Vc,
};

use crate::{context::WebsocketContext, invalidation::WebsocketMessage};

pub async fn identify(
    // TODO make identify payload_data (IdentifyPayloadData) partial
    payload_data: Vc<IdentifyPayloadData>,
    tt: Arc<dyn TurboTasksApi>,
    ctx: WebsocketContext,
) -> Result<()> {
    let reason = WebsocketMessage {
        opcode: OpCode::Identify,
        event: None,
        hide: false,
    };

    run_once_with_reason(tt.clone(), reason, async move {
        let mut write = ctx
            .api
            .write
            .try_lock()
            .expect("failed to lock `write` stream");

        let partial_payload_data = payload_data.await?.clone_value();

        let payload = Payload {
            op: OpCode::Identify,
            d: PayloadData::from(SendEvents::from(IdentifyPayloadData {
                properties: ConnectionProperties {
                    os: std::env::consts::OS.to_string(),
                    browser: "fujinoki".to_string(),
                    device: "fujinoki".to_string(),
                },
                ..partial_payload_data
            }))
            .into(),
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
