// Some of this file is ripped from:
// https://github.com/vercel/turbo/blob/main/crates/turbopack-cli/src/arguments.rs
// This file is MPL-2.0 licensed, as per the original file.

use std::{
    path::{Path, PathBuf},
    process,
};

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use fujinoki_cli_utils::issue::IssueSeverityCliOption;
use fujinoki_core::get_version;
use tracing::error;

#[derive(Debug, Parser)]
#[clap(author, version, about = "Framework for building Discord bots", long_about = None)]
#[clap(disable_help_subcommand = true)]
#[clap(arg_required_else_help = true)]
#[clap(disable_version_flag = true)]
#[clap(allow_external_subcommands = false)]
pub struct Arguments {
    #[clap(long, short = 'v', global = true)]
    pub version: bool,
    /// Force a check for a new version
    #[clap(long, global = true, hide = true)]
    pub check_for_update: bool,
    /// Disable the update notification
    #[clap(long, global = true, hide = true)]
    pub no_update_notifier: bool,
    #[clap(subcommand)]
    pub command: Option<Command>,
}

impl Arguments {
    pub fn parse() -> Result<Self> {
        let clap_args = match Arguments::try_parse() {
            Ok(args) => args,
            // Don't use error logger when displaying help text
            Err(e)
                if matches!(
                    e.kind(),
                    clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
                ) =>
            {
                let _ = e.print();
                process::exit(0);
            }
            Err(e) if e.use_stderr() => {
                let err_str = e.to_string();
                // A cleaner solution would be to implement our own clap::error::ErrorFormatter
                // but that would require copying the default formatter just to remove this
                // line: https://docs.rs/clap/latest/src/clap/error/format.rs.html#100
                error!(
                    "{}",
                    err_str.strip_prefix("error: ").unwrap_or(err_str.as_str())
                );
                let _ = e.print();
                process::exit(1);
            }
            // If the clap error shouldn't be printed to stderr it indicates help text
            Err(e) => {
                let _ = e.print();
                process::exit(0);
            }
        };

        // We have to override the --version flag because we use `get_version`
        // instead of a hard-coded version or the crate version
        if clap_args.version {
            println!("{}", get_version());
            process::exit(0);
        }

        Ok(clap_args)
    }

    /// The directory of the application. see [CommonArguments]::dir
    pub fn dir(&self) -> Option<&Path> {
        if let Some(command) = self.command.as_ref() {
            match command {
                Command::Upgrade(args) => args.common.dir.as_deref(),
                Command::Dev(args) => args.common.dir.as_deref(),
                Command::Build(args) => args.common.dir.as_deref(),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn should_trace(&self) -> bool {
        if let Some(command) = self.command.as_ref() {
            return match command {
                #[cfg(debug_assertions)]
                Command::Dev { .. } | &Command::Build { .. } => true,
                _ => false,
            };
        }

        return false;
    }
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Get the path to the binary
    Bin {},
    /// Upgrade to the latest version
    Upgrade(UpgradeArguments),
    Dev(DevArguments),
    Build(BuildArguments),
}

#[derive(Debug, Args)]
pub struct CommonArguments {
    /// The directory of the application.
    /// If no directory is provided, the current directory will be used.
    #[clap(short, long, value_parser)]
    pub dir: Option<PathBuf>,

    /// The root directory of the project. Nothing outside of this directory
    /// can be accessed. e. g. the monorepo root.
    /// If no directory is provided, `dir` will be used.
    #[clap(long, value_parser)]
    pub root: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct TurboArguments {
    /// Filter by issue severity.
    #[clap(short, long)]
    pub log_level: Option<IssueSeverityCliOption>,

    /// Show all log messages without limit.
    #[clap(long)]
    pub show_all: bool,

    /// Expand the log details.
    #[clap(long)]
    pub log_detail: bool,

    /// Whether to enable full task stats recording in Turbo Engine.
    #[clap(long)]
    pub full_stats: bool,

    /// Enable experimental garbage collection with the provided memory limit in
    /// MB.
    #[clap(long)]
    pub memory_limit: Option<usize>,
}

#[derive(Debug, Args)]
pub struct UpgradeArguments {
    #[clap(flatten)]
    pub common: CommonArguments,

    /// Upgrade to the canary build
    #[clap(long)]
    pub canary: bool,

    /// Dry run the upgrade
    #[clap(long)]
    pub dry_run: bool,
}

#[derive(Debug, Args)]
#[clap(author, version, about, long_about = None)]
pub struct DevArguments {
    #[clap(flatten)]
    pub common: CommonArguments,
    #[clap(flatten)]
    pub turbo: TurboArguments,
}

#[derive(Debug, Args)]
#[clap(author, version, about, long_about = None)]
pub struct BuildArguments {
    #[clap(flatten)]
    pub common: CommonArguments,
    #[clap(flatten)]
    pub turbo: TurboArguments,

    /// Don't minify build output.
    #[clap(long)]
    pub no_minify: bool,
}
