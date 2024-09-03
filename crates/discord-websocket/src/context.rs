use std::{collections::VecDeque, sync::Arc, time::Duration};

use anyhow::Result;
use futures::{stream::{SplitSink, SplitStream}, StreamExt};
use serde_json::Value as JsonValue;
use tokio::{net::TcpStream, sync::Mutex, task::JoinHandle, time::Instant};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};
use turbopack_binding::turbo::{tasks as turbo_tasks, tasks::TurboTasksApi};

pub type WebsocketStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Clone)]
pub struct WebsocketApi {
    pub write: Arc<Mutex<SplitSink<WebsocketStream, Message>>>,
    pub read: Arc<Mutex<SplitStream<WebsocketStream>>>,
}

impl WebsocketApi {
    pub fn new(gateway: WebsocketStream) -> Self {
        let (write, read) = gateway.split();

        Self {
            write: Arc::new(Mutex::new(write)),
            read: Arc::new(Mutex::new(read)),
        }
    }
}

#[turbo_tasks::value(shared, cell = "new", serialization = "none", eq = "manual")]
#[derive(Clone)]
pub struct WebsocketContext {
    pub debug: bool,
    #[turbo_tasks(trace_ignore, debug_ignore)]
    pub turbo_tasks: Arc<dyn TurboTasksApi>,
    #[turbo_tasks(trace_ignore, debug_ignore)]
    pub ongoing_side_effects: Arc<Mutex<VecDeque<Arc<Mutex<Option<JoinHandle<Result<()>>>>>>>>,
    #[turbo_tasks(trace_ignore, debug_ignore)]
    pub api: WebsocketApi,
    #[turbo_tasks(trace_ignore, debug_ignore)]
    pub sequence: Arc<Mutex<Option<u16>>>,
    #[turbo_tasks(trace_ignore, debug_ignore)]
    pub heartbeat_interval: Arc<Mutex<Option<Duration>>>,
    #[turbo_tasks(trace_ignore, debug_ignore)]
    pub last_heartbeat: Arc<Mutex<Instant>>,
    #[turbo_tasks(trace_ignore, debug_ignore)]
    pub heartbeat_ack: Arc<Mutex<bool>>,
    #[turbo_tasks(trace_ignore, debug_ignore)]
    pub session_id: Arc<Mutex<Option<String>>>,
    #[turbo_tasks(trace_ignore, debug_ignore)]
    pub resume_gateway_url: Arc<Mutex<Option<String>>>,
    #[turbo_tasks(trace_ignore, debug_ignore)]
    pub clean_client_data: Arc<Mutex<Option<JsonValue>>>,
}
