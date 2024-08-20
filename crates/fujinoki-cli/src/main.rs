#![feature(panic_info_message)]
#![feature(future_join)]
#![feature(min_specialization)]
#![feature(arbitrary_self_types)]

use std::{env::current_exe, panic, path::Path, thread};

use anyhow::{anyhow, Context, Result};
use fujinoki_cli::{
    arguments::{Arguments, Command},
    build, dev,
    panic_handler::panic_handler,
    register,
    tracing_presets::{
        TRACING_DISCORD_TARGETS, TRACING_FUJINOKI_TARGETS, TRACING_OVERVIEW_TARGETS,
        TRACING_TURBOPACK_TARGETS, TRACING_TURBO_TASKS_TARGETS,
    },
    upgrade,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use turbopack_binding::{
    turbo::malloc::TurboMalloc,
    turbopack::{
        core::error::PrettyPrintError,
        trace_server::start_turbopack_trace_server,
        trace_utils::{exit::ExitHandler, raw_trace::RawTraceLayer, trace_writer::TraceWriter},
    },
};

#[global_allocator]
static ALLOC: TurboMalloc = TurboMalloc;

pub fn main() {
    panic::set_hook(Box::new(panic_handler));

    let args = Arguments::parse().unwrap();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .on_thread_stop(|| {
            TurboMalloc::thread_stop();
        })
        .build()
        .unwrap()
        .block_on(main_inner(args))
        .inspect_err(|err| {
            eprintln!("{}", PrettyPrintError(&err));
        })
        .unwrap();
}

async fn main_inner(args: Arguments) -> Result<()> {
    let exit_handler = ExitHandler::listen();

    let trace = std::env::var("TURBOPACK_TRACING")
        .and_then(|res| match args.should_trace() {
            true => Ok(res),
            false => Err(std::env::VarError::NotPresent),
        })
        .ok();
    if let Some(mut trace) = trace {
        // Trace presets
        match trace.as_str() {
            "overview" | "1" => {
                trace = TRACING_OVERVIEW_TARGETS.join(",");
            }
            "fujinoki" => {
                trace = TRACING_FUJINOKI_TARGETS.join(",");
            }
            "discord" => {
                trace = TRACING_DISCORD_TARGETS.join(",");
            }
            "turbopack" => {
                trace = TRACING_TURBOPACK_TARGETS.join(",");
            }
            "turbo-tasks" => {
                trace = TRACING_TURBO_TASKS_TARGETS.join(",");
            }
            _ => {}
        }

        let subscriber = Registry::default();

        let subscriber = subscriber.with(EnvFilter::builder().parse(trace).unwrap());

        let internal_dir = args
            .dir()
            .unwrap_or_else(|| Path::new("."))
            .join(".turbopack");
        std::fs::create_dir_all(&internal_dir)
            .context("Unable to create .turbopack directory")
            .unwrap();
        let trace_file = internal_dir.join("trace.log");
        let trace_writer = std::fs::File::create(trace_file.clone()).unwrap();
        let (trace_writer, guard) = TraceWriter::new(trace_writer);
        let subscriber = subscriber.with(RawTraceLayer::new(trace_writer));

        exit_handler.on_exit(async move {
            tokio::task::spawn_blocking(move || drop(guard))
                .await
                .unwrap()
        });

        let trace_server = std::env::var("TURBOPACK_TRACE_SERVER").ok();
        if trace_server.is_some() {
            thread::spawn(move || {
                start_turbopack_trace_server(trace_file);
            });
            println!("Turbopack trace server started. View trace at https://turbo-trace-viewer.vercel.app/");
        }

        subscriber.init();
    }

    register();

    if let Some(command) = args.command.as_ref() {
        match command {
            Command::Bin {} => {
                let path = current_exe()
                    .map_err(|e| anyhow!("could not get path to binary: {}", e))
                    .unwrap();
                println!("{}", path.to_string_lossy());
            }
            Command::Upgrade(args) => upgrade::install_latest_build(args).await?,
            Command::Dev(args) => dev::start_server(args, exit_handler).await?,
            Command::Build(args) => build::build(args).await?,
        };
    }

    Ok(())
}
