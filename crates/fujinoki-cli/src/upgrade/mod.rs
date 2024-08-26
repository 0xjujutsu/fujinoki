// Display ripped from https://github.com/oven-sh/bun/blob/main/src/cli/upgrade_command.zig
// Logic ripped from https://github.com/vercel/turbo/blob/turbopack-240801.2/packages/turbo-codemod/src/commands/migrate/steps/getTurboUpgradeCommand.ts

use anyhow::Result;
use console::style;
use fujinoki_core::{get_version, DISPLAY_NAME, GITHUB_REPO, NPM_PACKAGE};
use turborepo_binding::turborepo::{
    path::AbsoluteSystemPath,
    repository::{package_json::PackageJson, package_manager::PackageManager},
};

use crate::{arguments::UpgradeArguments, util::normalize_dirs};

mod npm;

pub struct Command {
    pub bin: String,
    pub args: Vec<String>,
    pub dry_run: bool,
}

impl Command {
    pub fn new(bin: String) -> Command {
        Command {
            bin,
            args: vec![],
            dry_run: false,
        }
    }

    pub fn args<S: AsRef<str>, V: AsRef<[S]>>(mut self, args: V) -> Self {
        self.args
            .extend(args.as_ref().iter().map(|s| s.as_ref().to_string()));
        self
    }

    pub fn dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    pub fn execute(self, error: String) {
        let mut cmd = std::process::Command::new(self.bin);
        cmd.args(&self.args);
        if self.dry_run {
            return;
        }
        let status = cmd.status();
        assert!(status.is_ok(), "{}", error);
    }
}

// TODO use turbo_tasks to do this faster when there will be disk cache
pub async fn install_latest_build(args: &UpgradeArguments) -> Result<()> {
    let normalized_dirs = normalize_dirs(&args.common.dir, &args.common.root).unwrap();
    let to = if args.canary { "canary" } else { "latest" };

    let current_version = get_version();
    let version = if let Some(version) = npm::get_latest_version(NPM_PACKAGE, Some(to)).await? {
        version
    } else {
        eprintln!(
            "{} - Failed to fetch the {to} tag of {DISPLAY_NAME} from npm registry",
            style("error").red()
        );
        return Ok(());
    };

    if version.clone().eq(current_version) {
        eprintln!(
            "You're already on the latest version of {DISPLAY_NAME} {}",
            style(format!("(which is v{})", current_version)).dim()
        );
        return Ok(());
    }

    let package_json = PackageJson::load(
        &(AbsoluteSystemPath::new(&normalized_dirs.project_dir)?.join_component("package.json")),
    )?;
    let package_manager = PackageManager::detect_package_manager(
        &AbsoluteSystemPath::new(&normalized_dirs.project_dir).unwrap(),
    )?;
    let package_manager_exe_path = which::which_global(package_manager.to_string())?;
    let package_manager_bin = package_manager_exe_path.parent().unwrap().to_path_buf();

    let versioned_string = format!("{NPM_PACKAGE}@{}", to);
    let versioned = versioned_string.as_str();

    if let Ok(global_bin) = which::which_global(NPM_PACKAGE) {
        let should_global_install = global_bin.starts_with(package_manager_bin);

        if should_global_install {
            let global_install_args = match package_manager {
                PackageManager::Yarn | PackageManager::Berry => ["global", "add", versioned],
                PackageManager::Npm => ["install", versioned, "--global"],
                PackageManager::Bun
                | PackageManager::Pnpm
                | PackageManager::Pnpm6
                | PackageManager::Pnpm9 => ["add", versioned, "--global"],
            };

            Command::new(package_manager.to_string())
                .args(global_install_args)
                .dry_run(args.dry_run)
                .execute(format!(
                    "{} - Failed to upgrade {DISPLAY_NAME}, please try again. {}",
                    style("error").red(),
                    style(format!("(v{} → {})", current_version, version.clone())).dim()
                ));
        }
    } else {
        let root_path = AbsoluteSystemPath::new(normalized_dirs.root_dir.as_str())?;
        let project_path = AbsoluteSystemPath::new(normalized_dirs.project_dir.as_str())?;

        let workspace_globs = if package_json.other.contains_key("workspaces")
            | package_manager.to_string().starts_with("pnpm")
        {
            PackageManager::get_workspace_globs(&package_manager, root_path).map(Some)?
        } else {
            None
        };

        let is_dev_dependecy = if let Some(dev_dependecies) = package_json.dev_dependencies {
            dev_dependecies.contains_key(NPM_PACKAGE)
        } else if let Some(_) = package_json.dependencies {
            false
        } else {
            eprintln!(
                "`{NPM_PACKAGE}` {} - {}",
                style("error").red(),
                "is not listed in `dependencies`or `devDependencies` in your package.json."
            );
            return Ok(());
        };

        let is_workspace = if let Some(workspace_globs) = workspace_globs {
            workspace_globs.target_is_workspace(root_path, project_path)?
        } else {
            false
        };

        let empty = "";
        let local_install_args = match package_manager {
            PackageManager::Yarn => [
                "add",
                versioned,
                is_dev_dependecy.then(|| "--save-dev").unwrap_or(empty),
                empty,
            ],
            PackageManager::Berry => [
                "add",
                versioned,
                is_dev_dependecy.then(|| "--save-dev").unwrap_or(empty),
                is_workspace.then(|| "-W").unwrap_or(empty),
            ],
            PackageManager::Npm => [
                "install",
                versioned,
                is_dev_dependecy.then(|| "--save-dev").unwrap_or(empty),
                empty,
            ],
            PackageManager::Bun
            | PackageManager::Pnpm
            | PackageManager::Pnpm6
            | PackageManager::Pnpm9 => [
                "add",
                versioned,
                is_dev_dependecy.then(|| "--save-dev").unwrap_or(empty),
                is_workspace.then(|| "-w").unwrap_or(empty),
            ],
        };
        let local_install_args = local_install_args
            .iter()
            .filter(|arg| !arg.is_empty())
            .collect::<Vec<&&str>>();

        Command::new(package_manager.to_string())
            .args(local_install_args)
            .dry_run(args.dry_run)
            .execute(format!(
                "{} - Failed to upgrade {DISPLAY_NAME}, please try again. {}",
                style("error").red(),
                style(format!("(v{} → {})", current_version, version.clone())).dim()
            ));
    }

    if args.canary {
        println!(
            "Upgraded.\n\n{}",
            style(format!("Welcome to {DISPLAY_NAME}'s latest canary build!"))
                .green()
                .bold()
        )
    } else {
        println!(
            "Upgraded.\n\n{}\n\nWhat's new:\n\t {}\n\nChangelog:\n\t {}",
            style(format!("Welcome to {DISPLAY_NAME} v{version}!"))
                .green()
                .bold(),
            style(format!(
                "https://github.com/{GITHUB_REPO}/releases/tag/{prefix}-{version}",
                prefix = NPM_PACKAGE,
            ))
            .blue(),
            style(format!(
                "https://github.com/{GITHUB_REPO}/comapre/{prefix}-{current_version}...{prefix}-{version}",
                prefix = NPM_PACKAGE,
            ))
            .blue()
        )
    }

    Ok(())
}
