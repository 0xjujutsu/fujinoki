#![feature(arbitrary_self_types)]
#![feature(async_closure)]

use std::{collections::VecDeque, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use discord_api::gateway::{OpCode, Payload, PayloadToString};
use fujinoki_core::config::FujinokiConfig;
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde_json::{json, Value as JsonValue};
use source::ContentSourceData;
use tokio::{
    net::TcpStream,
    sync::Mutex,
    task::JoinHandle,
    time::{sleep, Instant},
};
use tokio_tungstenite::{
    connect_async_with_config,
    tungstenite::{protocol::WebSocketConfig, Message},
    MaybeTlsStream, WebSocketStream,
};
use turbopack_binding::{
    turbo::{
        tasks as turbo_tasks,
        tasks::{run_once, TurboTasksApi, Vc},
    },
    turbopack::{
        core::issue::{handle_issues, IssueExt, IssueReporter, IssueSeverity},
        node::debug::should_debug,
        trace_utils::exit::ExitHandler,
    },
};
use url::Url;

use crate::{events::WebsocketEvents, issue::WebsocketIssue};

pub mod discord;
mod events;
pub mod invalidation;
pub mod issue;
pub mod source;
mod util;

pub trait SourceProvider: Send + Clone + 'static {
    /// must call a turbo-tasks function internally
    fn get_source(&self) -> Vc<ContentSourceData>;
}

impl<T> SourceProvider for T
where
    T: FnOnce() -> Vc<ContentSourceData> + Send + Clone + 'static,
{
    fn get_source(&self) -> Vc<ContentSourceData> {
        self.clone()()
    }
}

pub type WebsocketStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Clone)]
pub struct WebsocketApi {
    pub write: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
    pub read: Arc<Mutex<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
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
    pub config: Vc<FujinokiConfig>,
    #[turbo_tasks(trace_ignore, debug_ignore)]
    turbo_tasks: Arc<dyn TurboTasksApi>,
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

pub struct Websocket {
    tt: Arc<dyn TurboTasksApi>,
    get_issue_reporter: Arc<dyn Fn() -> Vc<Box<dyn IssueReporter>> + Send + Sync>,
    config: Vc<FujinokiConfig>,
}

impl Websocket {
    pub fn new(
        tt: Arc<dyn TurboTasksApi>,
        config: Vc<FujinokiConfig>,
        get_issue_reporter: Arc<dyn Fn() -> Vc<Box<dyn IssueReporter>> + Send + Sync>,
    ) -> Self {
        Self {
            tt,
            get_issue_reporter,
            config,
        }
    }

    // TODO use turbo_tasks span macros for better trace logs
    pub async fn serve(
        self,
        source_provider: impl SourceProvider + Sync,
        exit_handler: Option<Arc<ExitHandler>>,
        gateway: WebsocketStream,
    ) -> Result<()> {
        let get_issue_reporter = self.get_issue_reporter.clone();

        let ctx = WebsocketContext {
            turbo_tasks: self.tt.clone(),
            config: self.config,
            api: WebsocketApi::new(gateway),
            debug: should_debug("websocket"),
            ongoing_side_effects: Arc::new(Mutex::new(VecDeque::<
                Arc<Mutex<Option<JoinHandle<Result<()>>>>>,
            >::with_capacity(16))),
            session_id: Arc::new(Mutex::new(None)),
            resume_gateway_url: Arc::new(Mutex::new(None)),
            sequence: Arc::new(Mutex::new(None)),
            heartbeat_interval: Arc::new(Mutex::new(None)),
            last_heartbeat: Arc::new(Mutex::new(Instant::now())),
            heartbeat_ack: Arc::new(Mutex::new(true)),
            clean_client_data: Arc::new(Mutex::new(None)),
        };
        // TODO rename `events` to `message_handler` (same with struct name)
        let events = WebsocketEvents::new(self.tt.clone(), ctx.clone(), get_issue_reporter);

        discord::identify(
            self.tt.clone(),
            self.get_issue_reporter.clone(),
            ctx.clone(),
        )
        .await?;

        // TODO move exit handler to separate function
        if let Some(exit_handler) = exit_handler {
            let write = ctx.api.write.clone();

            exit_handler.on_exit(async move {
                let mut write = write.try_lock().expect("failed to lock `write` stream");
                let payload = Payload {
                    op: OpCode::PresenceUpdate,
                    d: Some(json!({
                        "afk": false,
                        "status": "invisible",
                        "activities": []
                    })),
                    s: None,
                    t: None,
                };

                write
                    .send(payload.to_string().into())
                    .await
                    .context("send presence update")
                    .unwrap();
                write.flush().await.context("flush websocket").unwrap();
                write.close().await.context("close websocket").unwrap();
            })
        };

        self.serve_inner(source_provider, ctx, events).await
    }

    async fn serve_inner(
        &self,
        source_provider: impl SourceProvider + Sync,
        ctx: WebsocketContext,
        events: WebsocketEvents,
    ) -> Result<()> {
        loop {
            // Wait until all ongoing side effects are completed
            // We only need to wait for the ongoing side effects that were started
            // before this request. Later added side effects are not relevant for this.
            let current_ongoing_side_effects = {
                // Cleanup the ongoing_side_effects list
                let mut guard = ctx.ongoing_side_effects.lock().await;
                while let Some(front) = guard.front() {
                    let Ok(front_guard) = front.try_lock() else {
                        break;
                    };
                    if front_guard.is_some() {
                        break;
                    }
                    drop(front_guard);
                    guard.pop_front();
                }
                // Get a clone of the remaining list
                (*guard).clone()
            };
            // Wait for the side effects to complete
            for side_effect_mutex in current_ongoing_side_effects {
                let mut guard = side_effect_mutex.lock().await;
                if let Some(join_handle) = guard.take() {
                    join_handle.await??;
                }
                drop(guard);
            }

            discord::heartbeat(ctx.clone(), false).await?;

            let read = ctx.api.read.clone();
            let mut read = read.try_lock().expect("failed to lock `read` stream");

            // Make sure this is formatted correctly, the macro doesn't let the formatter do
            // its job properly
            tokio::select! {
                Some(message) = read.next() => match message {
                    Ok(Message::Text(message)) => events.text(
                            source_provider.clone(),
                            message,
                        ).await?,
                    Ok(Message::Binary(_)) => todo!("Message::Binary"),
                    Ok(Message::Ping(_)) => todo!("Message::Pong"),
                    Ok(Message::Pong(_)) => todo!("Message::Pong"),
                    Ok(Message::Close(message)) => events.close(
                        source_provider.clone(),
                        message,
                    ).await?,
                    Ok(Message::Frame(_)) => todo!("Message::Frame"),
                    Err(err) => {
                        let get_issue_reporter = self.get_issue_reporter.clone();
                        let source_provider = source_provider.clone();

                        run_once(self.tt.clone(), async move {
                            let issue_reporter = get_issue_reporter();
                            let source = source_provider.get_source();
                            let resolved_source = source.resolve_strongly_consistent().await?;

                            handle_issues(
                                source,
                                issue_reporter,
                                IssueSeverity::Fatal.cell(),
                                None,
                                Some("get source")
                            )
                            .await?;

                            let issue = WebsocketIssue {
                                path: resolved_source.await?.project_path,
                                title: match err.to_string().len() > 0 {
                                    true => err.to_string().into(),
                                    false => "Unknown error occurred".into(),
                                },
                                description: None,

                            }.cell();
                            issue.emit();

                            handle_issues(
                                issue,
                                issue_reporter,
                                IssueSeverity::Fatal.cell(),
                                None,
                                None
                            )
                            .await?;

                            Ok(())
                        }).await?
                    }
                },
                _ = sleep(Duration::from_secs(0)) => { },
            }
        }
    }
}

pub async fn connect_to_gateway(url: Option<String>, version: i8) -> Result<WebsocketStream> {
    let mut url = Url::parse(
        url.unwrap_or("wss://gateway.discord.gg".to_string())
            .as_str(),
    )
    .context("Failed to build gateway url")?;
    url.set_query(Some(&format!("v={}", version)));

    connect_async_with_config(
        url,
        Some(WebSocketConfig {
            max_frame_size: None,
            max_message_size: None,
            max_send_queue: None,
            accept_unmasked_frames: true,
        }),
        true,
    )
    .await
    .map(|(gateway, _)| gateway)
    .context("Failed to connect to gateway")
}

pub fn register() {
    discord_api::register();
    fujinoki_node::register();
    fujinoki_core::register();
    turbopack_binding::turbo::tasks_env::register();
    turbopack_binding::turbo::tasks_bytes::register();
    turbopack_binding::turbopack::core::register();
    turbopack_binding::turbopack::node::register();
    include!(concat!(env!("OUT_DIR"), "/register.rs"));
}

#[cfg(all(feature = "native-tls", feature = "rustls-tls"))]
compile_error!("You can't enable both `native-tls` and `rustls-tls`");

#[cfg(all(not(feature = "native-tls"), not(feature = "rustls-tls")))]
compile_error!("You have to enable one of the TLS backends: `native-tls` or `rustls-tls`");
