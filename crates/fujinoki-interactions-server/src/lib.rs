#![feature(min_specialization)]
#![feature(trait_alias)]
#![feature(array_chunks)]
#![feature(iter_intersperse)]
#![feature(str_split_remainder)]
#![feature(arbitrary_self_types)]

// mod http;
// pub mod introspect;
mod invalidation;

use std::{
    collections::VecDeque,
    future::Future,
    net::{SocketAddr, TcpListener},
    pin::Pin,
    sync::Arc,
    time::Instant,
};

use anyhow::{Context, Result};
use hyper::{
    server::{conn::AddrIncoming, Builder},
    service::{make_service_fn, service_fn},
    Request, Response, Server,
};
use parking_lot::Mutex;
use socket2::{Domain, Protocol, Socket, Type};
use tokio::task::JoinHandle;
use tracing::{event, info_span, Instrument, Level, Span};
use turbopack_binding::{
    turbo::tasks::{
        self as turbo_tasks, run_once_with_reason, trace::TraceRawVcs, util::FormatDuration,
        TurboTasksApi, Vc,
    },
    turbopack::core::{error::PrettyPrintError, issue::IssueReporter},
};

use crate::invalidation::{ServerRequest, ServerRequestSideEffects};

#[derive(TraceRawVcs, Debug)]
pub struct InteractionsServerBuilder {
    #[turbo_tasks(trace_ignore)]
    pub addr: SocketAddr,
    #[turbo_tasks(trace_ignore)]
    server: Builder<AddrIncoming>,
}

#[derive(TraceRawVcs)]
pub struct InteractionsServer {
    #[turbo_tasks(trace_ignore)]
    pub addr: SocketAddr,
    #[turbo_tasks(trace_ignore)]
    pub future: Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>,
}

impl InteractionsServer {
    pub fn listen(addr: SocketAddr) -> Result<InteractionsServerBuilder, anyhow::Error> {
        // This is annoying. The hyper::Server doesn't allow us to know which port was
        // bound (until we build it with a request handler) when using the standard
        // `server::try_bind` approach. This is important when binding the `0` port,
        // because the OS will remap that to an actual free port, and we need to know
        // that port before we build the request handler. So we need to construct a
        // real TCP listener, see if it bound, and get its bound address.
        let socket = Socket::new(Domain::for_address(addr), Type::STREAM, Some(Protocol::TCP))
            .context("unable to create socket")?;
        // Allow the socket to be reused immediately after closing. This ensures that
        // the dev server can be restarted on the same address without a buffer time for
        // the OS to release the socket.
        // https://docs.microsoft.com/en-us/windows/win32/winsock/using-so-reuseaddr-and-so-exclusiveaddruse
        #[cfg(not(windows))]
        let _ = socket.set_reuse_address(true);
        if matches!(addr, SocketAddr::V6(_)) {
            // When possible bind to v4 and v6, otherwise ignore the error
            let _ = socket.set_only_v6(false);
        }
        let sock_addr = addr.into();
        socket
            .bind(&sock_addr)
            .context("not able to bind address")?;
        socket.listen(128).context("not able to listen on socket")?;

        let listener: TcpListener = socket.into();
        let addr = listener
            .local_addr()
            .context("not able to get bound address")?;
        let server = Server::from_tcp(listener).context("Not able to start server")?;
        Ok(InteractionsServerBuilder { addr, server })
    }
}

impl InteractionsServerBuilder {
    pub fn serve(
        self,
        turbo_tasks: Arc<dyn TurboTasksApi>,
        // TODO source_provider: impl SourceProvider + Sync,
        get_issue_reporter: Arc<dyn Fn() -> Vc<Box<dyn IssueReporter>> + Send + Sync>,
    ) -> InteractionsServer {
        let ongoing_side_effects = Arc::new(Mutex::new(VecDeque::<
            Arc<tokio::sync::Mutex<Option<JoinHandle<Result<()>>>>>,
        >::with_capacity(16)));
        let make_svc = make_service_fn(move |_| {
            let tt = turbo_tasks.clone();
            // TODO let source_provider = source_provider.clone();
            let get_issue_reporter = get_issue_reporter.clone();
            let ongoing_side_effects = ongoing_side_effects.clone();
            async move {
                let handler = move |request: Request<hyper::Body>| {
                    let request_span = info_span!(parent: None, "request", name = ?request.uri());
                    let start = Instant::now();
                    let tt = tt.clone();
                    let get_issue_reporter = get_issue_reporter.clone();
                    let ongoing_side_effects = ongoing_side_effects.clone();
                    // TODO let source_provider = source_provider.clone();
                    let future = async move {
                        event!(parent: Span::current(), Level::DEBUG, "request start");
                        // Wait until all ongoing side effects are completed
                        // We only need to wait for the ongoing side effects that were started
                        // before this request. Later added side effects are not relevant for this.
                        let current_ongoing_side_effects = {
                            // Cleanup the ongoing_side_effects list
                            let mut guard = ongoing_side_effects.lock();
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
                        let reason = ServerRequest {
                            method: request.method().clone(),
                            uri: request.uri().clone(),
                        };
                        // let side_effects_reason = ServerRequestSideEffects {
                        //     method: request.method().clone(),
                        //     uri: request.uri().clone(),
                        // };
                        run_once_with_reason(tt.clone(), reason, async move {
                            // let issue_reporter = get_issue_reporter();

                            // if hyper_tungstenite::is_upgrade_request(&
                            // request) {
                            //     let uri = request.uri();
                            //     let path = uri.path();

                            //     if path == "/turbopack-hmr" {
                            //         let (response, websocket) =
                            //             hyper_tungstenite::upgrade(request,
                            // None)?;         let
                            // update_server =
                            //             UpdateServer::new(source_provider,
                            // issue_reporter);
                            //         update_server.run(&*tt, websocket);
                            //         return Ok(response);
                            //     }

                            //     println!("[404] {} (WebSocket)", path);
                            //     if path == "/_next/webpack-hmr" {
                            //         // Special-case requests to webpack-hmr
                            // as these are made by
                            //         // Next.js clients built
                            //         // without turbopack, which may be making
                            // requests in         //
                            // development.
                            //         println!(
                            //             "A non-turbopack next.js client is
                            // trying to connect."
                            //         );
                            //         println!(
                            //             "Make sure to reload/close any
                            // browser window which has \
                            //              been opened without --turbo."
                            //         );
                            //     }

                            //     return Ok(Response::builder()
                            //         .status(404)
                            //         .body(hyper::Body::empty())?);
                            // }

                            // let uri = request.uri();
                            // let path = uri.path().to_string();
                            // let source = source_provider.get_source();
                            // let resolved_source =
                            // source.resolve_strongly_consistent().await?;
                            // handle_issues(
                            //     source,
                            //     issue_reporter,
                            //     IssueSeverity::Fatal.cell(),
                            //     Some(&path),
                            //     Some("get source"),
                            // )
                            // .await?;
                            // let (response, side_effects) =
                            //     http::process_request_with_content_source(
                            //         resolved_source,
                            //         request,
                            //         issue_reporter,
                            //     )
                            //     .await?;
                            // let status = response.status().as_u16();
                            // let is_error =
                            // response.status().is_client_error()     ||
                            // response.status().is_server_error();
                            // let elapsed = start.elapsed();
                            // if is_error
                            //     || (cfg!(feature = "log_request_stats")
                            //         && elapsed > Duration::from_secs(1))
                            // {
                            //     println!(
                            //         "[{status}] {path} ({duration})",
                            //         duration = FormatDuration(elapsed)
                            //     );
                            // }
                            // if !side_effects.is_empty() {
                            //     let join_handle =
                            // tokio::spawn(run_once_with_reason(
                            //         tt.clone(),
                            //         side_effects_reason,
                            //         async move {
                            //             for side_effect in side_effects {
                            //                 side_effect.apply().await?;
                            //             }
                            //             Ok(())
                            //         },
                            //     ));
                            //     ongoing_side_effects.lock().
                            // push_back(Arc::new(
                            //         tokio::sync::Mutex::new(Some(join_handle)),
                            //     ));
                            // }
                            // Ok(response)
                            Ok(Response::builder().status(404).body(hyper::Body::empty())?)
                        })
                        .await
                    };
                    async move {
                        match future.await {
                            Ok(r) => Ok::<_, hyper::http::Error>(r),
                            Err(e) => {
                                println!(
                                    "[500] error ({}): {}",
                                    FormatDuration(start.elapsed()),
                                    PrettyPrintError(&e),
                                );
                                Ok(Response::builder()
                                    .status(500)
                                    .body(hyper::Body::from(format!("{}", PrettyPrintError(&e))))?)
                            }
                        }
                    }
                    .instrument(request_span)
                };
                anyhow::Ok(service_fn(handler))
            }
        });
        let server = self.server.serve(make_svc);

        InteractionsServer {
            addr: self.addr,
            future: Box::pin(async move {
                server.await?;
                Ok(())
            }),
        }
    }
}

pub fn register() {
    fujinoki_cli_utils::register();
    turbo_tasks::register();
    turbopack_binding::turbo::tasks_bytes::register();
    turbopack_binding::turbo::tasks_fs::register();
    turbopack_binding::turbopack::core::register();
    turbopack_binding::turbopack::ecmascript::register();
    include!(concat!(env!("OUT_DIR"), "/register.rs"));
}

#[cfg(all(feature = "native-tls", feature = "rustls-tls"))]
compile_error!("You can't enable both `native-tls` and `rustls-tls`");

#[cfg(all(not(feature = "native-tls"), not(feature = "rustls-tls")))]
compile_error!("You have to enable one of the TLS backends: `native-tls` or `rustls-tls`");
