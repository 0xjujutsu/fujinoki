use std::{future::Future, pin::Pin, sync::Arc};

use anyhow::{Context, Result};
use fujinoki_core::config::FujinokiConfig;
use fujinoki_websocket::{connect_to_gateway, SourceProvider, Websocket, WebsocketStream};
use turbopack_binding::{
    turbo::tasks::{self as turbo_tasks, trace::TraceRawVcs, TurboTasksApi, Vc},
    turbopack::{core::issue::IssueReporter, trace_utils::exit::ExitHandler},
};

#[derive(TraceRawVcs)]
pub struct DevServerBuilder {
    #[turbo_tasks(trace_ignore)]
    gateway: WebsocketStream,
}

#[derive(TraceRawVcs)]
pub struct DevServer {
    #[turbo_tasks(trace_ignore)]
    pub future: Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>,
}

impl DevServer {
    pub async fn connect(
        url: Option<String>,
        version: i8,
    ) -> Result<DevServerBuilder, anyhow::Error> {
        let gateway = connect_to_gateway(url, version)
            .await
            .context("Failed to connect to gateway")?;

        Ok(DevServerBuilder { gateway })
    }
}

impl DevServerBuilder {
    pub fn serve(
        self,
        turbo_tasks: Arc<dyn TurboTasksApi>,
        source_provider: impl SourceProvider + Sync,
        get_issue_reporter: Arc<dyn Fn() -> Vc<Box<dyn IssueReporter>> + Send + Sync>,
        exit_handler: Option<Arc<ExitHandler>>,
        config: Vc<FujinokiConfig>,
    ) -> DevServer {
        let websocket = Websocket::new(turbo_tasks.clone(), config, get_issue_reporter);
        let server = websocket.serve(source_provider, exit_handler, self.gateway);

        DevServer {
            future: Box::pin(async move {
                server.await?;
                Ok(())
            }),
        }
    }
}

pub fn register() {
    fujinoki_core::register();
    fujinoki_websocket::register();
    turbo_tasks::register();
    turbopack_binding::turbopack::core::register();
    include!(concat!(env!("OUT_DIR"), "/register.rs"));
}

#[cfg(all(feature = "native-tls", feature = "rustls-tls"))]
compile_error!("You can't enable both `native-tls` and `rustls-tls`");

#[cfg(all(not(feature = "native-tls"), not(feature = "rustls-tls")))]
compile_error!("You have to enable one of the TLS backends: `native-tls` or `rustls-tls`");
