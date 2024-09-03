use std::time::Duration;

use anyhow::{anyhow, Result};
use discord_api::gateway::opcode::OpCode;
use futures::SinkExt;
use serde_json::json;
use tokio::time::Instant;
use tokio_tungstenite::tungstenite::Message;

use crate::context::WebsocketContext;

pub async fn heartbeat(ctx: WebsocketContext, force: bool) -> Result<()> {
    let heartbeat_interval = ctx
        .heartbeat_interval
        .try_lock()
        .expect("failed to lock `heartbeat_interval`");

    if let Some(interval) = heartbeat_interval
        .map(|i| Some(Duration::from(i)))
        .unwrap_or(None)
    {
        let last_heartbeat = ctx.last_heartbeat;
        let elapsed = {
            let last_heartbeat = last_heartbeat
                .try_lock()
                .expect("failed to lock `last_heartbeat`");
            last_heartbeat.elapsed()
        };

        if force || elapsed >= interval {
            let mut heartbeat_ack = ctx
                .heartbeat_ack
                .try_lock()
                .expect("failed to lock `heartbeat_ack`");

            if !*heartbeat_ack && !force {
                return Err(anyhow!("last heartbeat was not acknowledged"));
            }

            let write = ctx.api.write.clone();
            let sequence = ctx.sequence.try_lock().expect("failed to lock `sequence`");

            let mut write = write.try_lock().expect("failed to lock `write` stream");
            let mut last_heartbeat = last_heartbeat
                .try_lock()
                .expect("failed to lock `last_heartbeat`");

            let payload = json!({
                "op": OpCode::Heartbeat,
                "d": *sequence,
            });

            return match write.send(Message::Text(payload.to_string())).await {
                Ok(_) => {
                    *last_heartbeat = Instant::now().into();
                    *heartbeat_ack = false;

                    Ok(())
                }
                Err(err) => Err(anyhow!("failed to send heartbeat to gateway: {err}")),
            };
        }
    }

    Ok(())
}
