use std::{
    collections::{HashMap, HashSet},
    env::{current_dir, var_os},
    path::PathBuf,
    process,
};

use clap::{arg, Command};

mod command;
mod publish;

use publish::{publish_workspace, run_bump, run_publish};

fn cli() -> Command {
    Command::new("xtask")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("npm")
                .about("Publish binaries to npm")
                .arg(arg!(<NAME> "the package to publish"))
                .arg(arg!(--nightly "publish nightly version"))
                .arg(arg!(--"dry-run" "dry run all operations"))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("workspace")
                .arg(arg!(--publish "publish npm packages in pnpm workspace"))
                .arg(arg!(--bump "bump new version for npm package in pnpm workspace"))
                .arg(arg!(--"dry-run" "dry run all operations"))
                .arg(arg!([NAME] "the package to bump"))
                .about("Manage packages in pnpm workspaces"),
        )
        .subcommand(
            Command::new("upgrade-swc").about("Upgrade all SWC dependencies to the latest version"),
        )
}

fn main() {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("npm", sub_matches)) => {
            let name = sub_matches
                .get_one::<String>("NAME")
                .expect("NAME is required");
            let nightly = sub_matches.get_flag("nightly");
            let dry_run = sub_matches.get_flag("dry-run");

            run_publish(name, nightly, dry_run);
        }
        Some(("workspace", sub_matches)) => {
            let is_bump = sub_matches.get_flag("bump");
            let is_publish = sub_matches.get_flag("publish");
            let dry_run = sub_matches.get_flag("dry-run");
            if is_bump {
                let names = sub_matches
                    .get_many::<String>("NAME")
                    .map(|names| names.cloned().collect::<HashSet<_>>())
                    .unwrap_or_default();
                run_bump(names, dry_run);
            }
            if is_publish {
                publish_workspace(dry_run);
            }
        }
        Some(("upgrade-swc", _)) => {
            let workspace_dir = var_os("CARGO_WORKSPACE_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| current_dir().unwrap());
            let cargo_lock_path = workspace_dir.join("../../Cargo.lock");
            let lock = cargo_lock::Lockfile::load(cargo_lock_path).unwrap();
            let swc_packages = lock
                .packages
                .iter()
                .filter(|p| {
                    p.name.as_str().starts_with("swc_")
                        || p.name.as_str() == "swc"
                        || p.name.as_str() == "testing"
                })
                .collect::<Vec<_>>();
            let only_swc_set = swc_packages
                .iter()
                .map(|p| p.name.as_str())
                .collect::<HashSet<_>>();
            let packages = lock
                .packages
                .iter()
                .map(|p| (format!("{}@{}", p.name, p.version), p))
                .collect::<HashMap<_, _>>();
            let mut queue = swc_packages.clone();
            let mut set = HashSet::new();
            while let Some(package) = queue.pop() {
                for dep in package.dependencies.iter() {
                    let ident = format!("{}@{}", dep.name, dep.version);
                    let package = *packages.get(&ident).unwrap();
                    if set.insert(ident) {
                        queue.push(package);
                    }
                }
            }
            let status = process::Command::new("cargo")
                .arg("upgrade")
                .arg("--workspace")
                .args(only_swc_set)
                .current_dir(&workspace_dir)
                .stdout(process::Stdio::inherit())
                .stderr(process::Stdio::inherit())
                .status()
                .expect("Running cargo upgrade failed");
            assert!(status.success());
            let status = process::Command::new("cargo")
                .arg("update")
                .args(set.iter().flat_map(|p| ["-p", p]))
                .current_dir(&workspace_dir)
                .stdout(process::Stdio::inherit())
                .stderr(process::Stdio::inherit())
                .status()
                .expect("Running cargo update failed");
            assert!(status.success());
        }
        _ => {
            panic!("Unknown command {:?}", matches.subcommand().map(|c| c.0));
        }
    }
}
