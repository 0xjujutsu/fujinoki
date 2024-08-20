use std::{
    env::current_dir,
    future::{join, Future},
    io::{stdout, Write},
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Result;
use console::style;
use fujinoki_cli_utils::issue::{ConsoleUi, LogOptions};
use fujinoki_core::config::FujinokiConfig;
use fujinoki_dev_server::DevServer;
use owo_colors::OwoColorize;
use turbopack_binding::{
    turbo::{
        malloc::TurboMalloc,
        tasks::{
            util::{FormatBytes, FormatDuration},
            TransientInstance, TurboTasks, UpdateInfo, Vc,
        },
        tasks_memory::MemoryBackend,
    },
    turbopack::{
        core::issue::{handle_issues, IssueReporter, IssueSeverity},
        trace_utils::exit::ExitHandler,
    },
};

use crate::{
    arguments::DevArguments,
    contexts::NodeEnv,
    dev::source::{get_project_path, source},
    util::{normalize_dirs, EntryRequest, NormalizedDirs},
};

pub(crate) mod source;

struct FujinokiDevServerBuilder {
    turbo_tasks: Arc<TurboTasks<MemoryBackend>>,
    project_dir: String,
    root_dir: String,
    entry_requests: Vec<EntryRequest>,
    issue_reporter: Option<Box<dyn IssueReporterProvider>>,
    log_level: IssueSeverity,
    show_all: bool,
    log_detail: bool,
    exit_handler: Option<Arc<ExitHandler>>,
}

impl FujinokiDevServerBuilder {
    pub fn new(
        turbo_tasks: Arc<TurboTasks<MemoryBackend>>,
        project_dir: String,
        root_dir: String,
    ) -> FujinokiDevServerBuilder {
        FujinokiDevServerBuilder {
            turbo_tasks,
            project_dir,
            root_dir,
            entry_requests: vec![],
            issue_reporter: None,
            log_level: IssueSeverity::Warning,
            show_all: false,
            log_detail: false,
            exit_handler: None,
        }
    }

    pub fn log_level(mut self, log_level: IssueSeverity) -> FujinokiDevServerBuilder {
        self.log_level = log_level;
        self
    }

    pub fn show_all(mut self, show_all: bool) -> FujinokiDevServerBuilder {
        self.show_all = show_all;
        self
    }

    pub fn log_detail(mut self, log_detail: bool) -> FujinokiDevServerBuilder {
        self.log_detail = log_detail;
        self
    }

    pub fn exit_handler(mut self, exit_handler: Arc<ExitHandler>) -> FujinokiDevServerBuilder {
        self.exit_handler = Some(exit_handler);
        self
    }

    pub async fn build(self) -> Result<DevServer> {
        let turbo_tasks = self.turbo_tasks;
        let project_dir = self.project_dir;
        let root_dir = self.root_dir;
        let show_all = self.show_all;
        let log_detail = self.log_detail;
        let log_args = TransientInstance::new(LogOptions {
            current_dir: current_dir().unwrap(),
            project_dir: PathBuf::from(project_dir.clone()),
            show_all,
            log_detail,
            log_level: self.log_level,
        });
        let entry_requests = TransientInstance::new(self.entry_requests);
        let tasks = turbo_tasks.clone();
        let issue_provider = self.issue_reporter.unwrap_or_else(|| {
            // Initialize a ConsoleUi reporter if no custom reporter was provided
            Box::new(move || Vc::upcast(ConsoleUi::new(log_args.clone())))
        });
        let issue_reporter_arc = Arc::new(move || issue_provider.get_issue_reporter());

        let root_dir_clone = root_dir.clone();
        let project_dir_clone = root_dir.clone();
        let issue_reporter_arc_clone = issue_reporter_arc.clone();
        let config = tasks
            .clone()
            .run_once::<Vc<FujinokiConfig>>(async move {
                let issue_reporter = issue_reporter_arc_clone();
                let project_path =
                    get_project_path(root_dir_clone.into(), project_dir_clone.into());
                let config_path = project_path.join("fujinoki.config.json".into());
                let config = FujinokiConfig::from_json(
                    config_path,
                    Some(NodeEnv::Development.to_string().into()),
                );

                handle_issues(
                    config,
                    issue_reporter,
                    IssueSeverity::Fatal.cell(),
                    None,
                    Some("get config"),
                )
                .await?;

                // TODO validate the config

                Ok(config)
            })
            .await?;

        let gateway = DevServer::connect(None, discord_api::VERSION).await?;

        let source = move || {
            source(
                root_dir.into(),
                project_dir.into(),
                entry_requests.clone(),
                config,
            )
        };

        Ok(gateway.serve(
            tasks.clone(),
            source,
            issue_reporter_arc,
            self.exit_handler,
            config,
        ))
    }
}

pub fn register() {
    fujinoki_cli_utils::register();
    fujinoki_dev_server::register();
    fujinoki_core::register();
    turbopack_binding::turbopack::core::register();
    turbopack_binding::turbopack::ecmascript_runtime::register();
    turbopack_binding::turbopack::env::register();
    turbopack_binding::turbopack::node::register();
    turbopack_binding::turbopack::nodejs::register();
}

pub async fn start_server(args: &DevArguments, exit_handler: &Arc<ExitHandler>) -> Result<()> {
    register();

    let start = Instant::now();

    #[cfg(feature = "tokio_console")]
    console_subscriber::init();

    let NormalizedDirs {
        project_dir,
        root_dir,
    } = normalize_dirs(&args.common.dir, &args.common.root).unwrap();

    let tt = TurboTasks::new(MemoryBackend::new(
        args.turbo
            .memory_limit
            .map_or(usize::MAX, |l| l * 1024 * 1024),
    ));

    let server = FujinokiDevServerBuilder::new(tt.clone(), project_dir.clone(), root_dir.clone())
        .log_detail(args.turbo.log_detail)
        .show_all(args.turbo.show_all)
        .log_level(
            args.turbo
                .log_level
                .map_or_else(|| IssueSeverity::Warning, |l| l.0),
        )
        .exit_handler(exit_handler.clone());

    let server = server.build().await?;

    println!("{} - started server", style("ready").green());

    let tt_clone = tt.clone();
    let stats_future = async move {
        if args.turbo.log_detail {
            println!(
                "{event_type} - initial compilation {start} ({memory})",
                event_type = "event".purple(),
                start = FormatDuration(start.elapsed()),
                memory = FormatBytes(TurboMalloc::memory_usage())
            );
        }

        let mut progress_counter = 0;
        loop {
            let update_future = profile_timeout(
                tt_clone.as_ref(),
                tt_clone.aggregated_update_info(Duration::from_millis(100), Duration::MAX),
            );

            if let Some(UpdateInfo {
                duration,
                tasks,
                reasons,
                ..
            }) = update_future.await
            {
                progress_counter = 0;

                if !args.turbo.show_all && reasons.to_string().contains("[hide]") {
                    continue;
                }

                match (args.turbo.log_detail, !reasons.is_empty()) {
                    (true, true) => {
                        println!(
                            "\x1b[2K{event_type} - {reasons} {duration} ({tasks} tasks, {memory})",
                            reasons = reasons.to_string().replace("[hide] ", ""),
                            event_type = "event".purple(),
                            duration = FormatDuration(duration),
                            tasks = tasks,
                            memory = FormatBytes(TurboMalloc::memory_usage())
                        );
                    }
                    (true, false) => {
                        println!(
                            "\x1b[2K{event_type} - compilation {duration} ({tasks} tasks, \
                             {memory})",
                            event_type = "event".purple(),
                            duration = FormatDuration(duration),
                            tasks = tasks,
                            memory = FormatBytes(TurboMalloc::memory_usage())
                        );
                    }
                    (false, true) => {
                        println!(
                            "\x1b[2K{event_type} - {reasons} {duration}",
                            reasons = reasons.to_string().replace("[hide] ", ""),
                            event_type = "event".purple(),
                            duration = FormatDuration(duration),
                        );
                    }
                    (false, false) => {
                        if duration > Duration::from_secs(1) {
                            println!(
                                "\x1b[2K{event_type} - compilation {duration}",
                                event_type = "event".purple(),
                                duration = FormatDuration(duration),
                            );
                        }
                    }
                }
            } else {
                progress_counter += 1;
                if args.turbo.log_detail {
                    print!(
                        "\x1b[2K{event_type} - updating for {progress_counter}s... ({memory})\r",
                        event_type = "event".purple(),
                        memory = FormatBytes(TurboMalloc::memory_usage())
                    );
                } else {
                    print!(
                        "\x1b[2K{event_type} - updating for {progress_counter}s...\r",
                        event_type = "event".purple(),
                    );
                }
                let _ = stdout().lock().flush();
            }
        }
    };

    join!(stats_future, async { server.future.await.unwrap() }).await;

    Ok(())
}

#[cfg(feature = "profile")]
// When profiling, exits the process when no new updates have been received for
// a given timeout and there are no more tasks in progress.
async fn profile_timeout<T>(tt: &TurboTasks<MemoryBackend>, future: impl Future<Output = T>) -> T {
    /// How long to wait in between updates before force-exiting the process
    /// during profiling.
    const PROFILE_EXIT_TIMEOUT: Duration = Duration::from_secs(5);

    futures::pin_mut!(future);
    loop {
        match tokio::time::timeout(PROFILE_EXIT_TIMEOUT, &mut future).await {
            Ok(res) => return res,
            Err(_) => {
                if tt.get_in_progress_count() == 0 {
                    std::process::exit(0)
                }
            }
        }
    }
}

#[cfg(not(feature = "profile"))]
fn profile_timeout<T>(
    _tt: &TurboTasks<MemoryBackend>,
    future: impl Future<Output = T>,
) -> impl Future<Output = T> {
    future
}

pub trait IssueReporterProvider: Send + Sync + 'static {
    fn get_issue_reporter(&self) -> Vc<Box<dyn IssueReporter>>;
}

impl<T> IssueReporterProvider for T
where
    T: Fn() -> Vc<Box<dyn IssueReporter>> + Send + Sync + Clone + 'static,
{
    fn get_issue_reporter(&self) -> Vc<Box<dyn IssueReporter>> {
        self()
    }
}
