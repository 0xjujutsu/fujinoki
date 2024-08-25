use std::{
    collections::{HashMap, HashSet},
    env, fs,
    path::PathBuf,
    process,
    str::FromStr,
    sync::Arc,
};

use owo_colors::OwoColorize;
use semver::{Prerelease, Version};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use turbopack_binding::swc::core::{
    base::{config::JsMinifyOptions, try_with_handler, BoolOrDataConfig, Compiler as SwcCompiler},
    common::{FileName, SourceMap, GLOBALS},
};

mod napi;
mod util;

use self::{
    napi::{create_cjs_binding, typegen::process_typedef},
    util::{default_empty_string, get_nightly_version},
};
use crate::command::Command;

const PLATFORM_LINUX_MUSL_X64: NpmSupportedPlatform = NpmSupportedPlatform {
    os: "linux",
    arch: "x64",
    rust_target: "x86_64-unknown-linux-musl",
};

const PLATFORM_LINUX_GNU_X64: NpmSupportedPlatform = NpmSupportedPlatform {
    os: "linux",
    arch: "x64",
    rust_target: "x86_64-unknown-linux-gnu",
};

const PLATFORM_DARWIN_X64: NpmSupportedPlatform = NpmSupportedPlatform {
    os: "darwin",
    arch: "x64",
    rust_target: "x86_64-apple-darwin",
};

const PLATFORM_DARWIN_ARM64: NpmSupportedPlatform = NpmSupportedPlatform {
    os: "darwin",
    arch: "arm64",
    rust_target: "aarch64-apple-darwin",
};

const PLATFORM_WIN32_X64: NpmSupportedPlatform = NpmSupportedPlatform {
    os: "win32",
    arch: "x64",
    rust_target: "x86_64-pc-windows-msvc",
};

<<<<<<< HEAD:xtask/src/publish.rs
const NPM_PACKAGES: &[NpmPackage] = &[NpmPackage {
    crate_name: "fujinoki-cli",
    name: "fujinoki",
    description: "Discord bot framework",
    kind: NpmPackageKind::Bin("fujinoki"),
    platform: &[
        PLATFORM_LINUX_X64,
        PLATFORM_DARWIN_X64,
        PLATFORM_DARWIN_ARM64,
        PLATFORM_WIN32_X64,
    ],
}];
=======
const NPM_PACKAGES: &[NpmPackage] = &[
    NpmPackage {
        crate_name: "fujinoki-cli",
        name: "fujinoki",
        description: "Discord bot framework",
        kind: NpmPackageKind::Bin("fujinoki"),
        platform: &[
            PLATFORM_LINUX_MUSL_X64,
            PLATFORM_DARWIN_X64,
            PLATFORM_DARWIN_ARM64,
            PLATFORM_WIN32_X64,
        ],
    },
    NpmPackage {
        crate_name: "discord-api-napi",
        name: "@fujinoki/discord-api",
        description: "Discord API bindings",
        kind: NpmPackageKind::Napi,
        platform: &[
            PLATFORM_LINUX_GNU_X64,
            PLATFORM_DARWIN_X64,
            PLATFORM_DARWIN_ARM64,
            PLATFORM_WIN32_X64,
        ],
    },
];
>>>>>>> 9f27a5b (feat(xtask): ci utils (#5)):xtask/src/publish/mod.rs

struct NpmSupportedPlatform {
    os: &'static str,
    arch: &'static str,
    rust_target: &'static str,
}

enum NpmPackageKind {
    Bin(&'static str),
<<<<<<< HEAD:xtask/src/publish.rs
    Napi(Option<&'static str>),
=======
    Napi,
>>>>>>> 9f27a5b (feat(xtask): ci utils (#5)):xtask/src/publish/mod.rs
}

struct NpmPackage {
    crate_name: &'static str,
    name: &'static str,
    description: &'static str,
    kind: NpmPackageKind,
    platform: &'static [NpmSupportedPlatform],
}

<<<<<<< HEAD:xtask/src/publish.rs
pub fn run_publish(name: &str, nightly: bool, dry_run: bool) {
=======
pub fn run_publish(name: &str, is_nightly: bool, dry_run: bool) {
>>>>>>> 9f27a5b (feat(xtask): ci utils (#5)):xtask/src/publish/mod.rs
    if let Some(pkg) = NPM_PACKAGES.iter().find(|p| p.crate_name == name) {
        let mut optional_dependencies = Vec::with_capacity(pkg.platform.len());
        let mut is_alpha = false;
        let mut is_beta = false;
        let mut is_canary = false;
<<<<<<< HEAD:xtask/src/publish.rs
        let version = match nightly {
            true => {
                let now = chrono::Local::now();
                let nightly_tag = Command::program("git")
                    .args(["tag", "-l", "nightly-*"])
                    .error_message("Failed to list nightly tags")
                    .output_string();
                dbg!(nightly_tag.clone());

                let latest_nightly_tag = nightly_tag
                    .lines()
                    .filter(|tag| tag.starts_with("nightly-"))
                    .max()
                    .unwrap_or("");

                let patch = if latest_nightly_tag.is_empty() {
                    0
                } else {
                    latest_nightly_tag
                        .split('.')
                        .last()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0)
                };

                format!("0.0.0-nightly.{}.{}", now.format("%y%m%d"), patch + 1)
            }
            false => {
                if let Ok(release_version) = env::var("RELEASE_VERSION") {
                    // node-file-trace@1.0.0-alpha.1
                    let release_tag_version = release_version
                        .trim()
                        .trim_start_matches(format!("{}@", pkg.name).as_str());
                    if let Ok(semver_version) = Version::parse(release_tag_version) {
                        is_alpha = semver_version.pre.contains("alpha");
                        is_beta = semver_version.pre.contains("beta");
                        is_canary = semver_version.pre.contains("canary");
                    };
                    release_tag_version.to_owned()
                } else {
                    format!(
                        "0.0.0-{}",
                        env::var("GITHUB_SHA")
                            .map(|mut sha| {
                                sha.truncate(7);
                                sha
                            })
                            .unwrap_or_else(|_| {
                                if let Ok(mut o) = process::Command::new("git")
                                    .args(["rev-parse", "--short", "HEAD"])
                                    .output()
                                    .map(|o| {
                                        String::from_utf8(o.stdout).expect("Invalid utf8 output")
                                    })
                                {
                                    o.truncate(7);
                                    return o;
                                }
                                panic!("Unable to get git commit sha");
                            })
                    )
                }
            }
        };
        let tag = if version.contains("nightly") {
=======
        let version = if is_nightly {
            get_nightly_version()
        } else {
            if let Ok(release_version) = env::var("RELEASE_VERSION") {
                // node-file-trace@1.0.0-alpha.1
                let release_tag_version = release_version
                    .trim()
                    .trim_start_matches(format!("{}@", pkg.name).as_str());
                if let Ok(semver_version) = Version::parse(release_tag_version) {
                    is_alpha = semver_version.pre.contains("alpha");
                    is_beta = semver_version.pre.contains("beta");
                    is_canary = semver_version.pre.contains("canary");
                };
                release_tag_version.to_owned()
            } else {
                format!(
                    "0.0.0-{}",
                    env::var("GITHUB_SHA")
                        .map(|mut sha| {
                            sha.truncate(7);
                            sha
                        })
                        .unwrap_or_else(|_| {
                            if let Ok(mut o) = process::Command::new("git")
                                .args(["rev-parse", "--short", "HEAD"])
                                .output()
                                .map(|o| String::from_utf8(o.stdout).expect("Invalid utf8 output"))
                            {
                                o.truncate(7);
                                return o;
                            }
                            panic!("Unable to get git commit sha");
                        })
                )
            }
        };
        let tag = if is_nightly {
>>>>>>> 9f27a5b (feat(xtask): ci utils (#5)):xtask/src/publish/mod.rs
            "nightly"
        } else if is_alpha {
            "alpha"
        } else if is_beta {
            "beta"
        } else if is_canary {
            "canary"
        } else {
            "latest"
        };
        let pkg_name_in_path = if pkg.name.starts_with("@fujinoki") {
            // @fujinoki/pkg -> pkg
            &pkg.name.replace("@fujinoki/", "")
        } else if pkg.name.starts_with("@") {
            // @scope/pkg -> scope-pkg
            &pkg.name.replace("@", "").replace("/", "-")
        } else {
            &pkg.name.to_string()
        };
        let current_dir = env::current_dir().expect("Unable to get current directory");
<<<<<<< HEAD:xtask/src/publish.rs
        let package_dir = current_dir.join("../../packages").join(pkg.name);
        let temp_dir = package_dir.join("npm");
        if let Ok(()) = fs::remove_dir_all(&temp_dir) {};
        fs::create_dir(&temp_dir).expect("Unable to create temporary npm directory");
        for platform in pkg.platform.iter() {
            match pkg.kind {
                NpmPackageKind::Bin(bin) => {
                    let bin_file_name = if platform.os == "win32" {
                        format!("{}.exe", bin)
                    } else {
                        bin.to_string()
                    };
                    let platform_package_name =
                        format!("{}-{}-{}", pkg.name, platform.os, platform.arch);
                    optional_dependencies.push(platform_package_name.clone());
                    let pkg_json = serde_json::json!({
                      "name": platform_package_name,
                      "version": version,
                      "description": pkg.description,
                      "os": [platform.os],
                      "cpu": [platform.arch],
                      "bin": {
                        bin: bin_file_name
                      }
                    });

                    let dir_name = format!("{}-{}-{}", pkg.crate_name, platform.os, platform.arch);
                    let target_dir = package_dir.join("npm").join(dir_name);
                    fs::create_dir(&target_dir)
                        .unwrap_or_else(|e| panic!("Unable to create dir: {:?}\n{e}", &target_dir));
                    fs::write(
                        target_dir.join("package.json"),
                        serde_json::to_string_pretty(&pkg_json).unwrap(),
                    )
                    .expect("Unable to write package.json");
                    let artifact_path = current_dir
                        .join("artifacts")
                        .join(format!("{}-{}", pkg.crate_name, platform.rust_target))
                        .join(&bin_file_name);
                    let dist_path = target_dir.join(&bin_file_name);
                    fs::copy(&artifact_path, &dist_path).unwrap_or_else(|e| {
                        panic!(
                            "Copy file from [{:?}] to [{:?}] failed: {e}",
                            artifact_path, dist_path
                        )
                    });
                    Command::program("npm")
                        .args(["publish", "--access", "public", "--tag", tag])
                        .error_message("Publish npm package failed")
                        .current_dir(target_dir)
                        .dry_run(dry_run)
                        .execute();
                }
                NpmPackageKind::Napi(napi) => todo!("napi impl"),
            }
=======
        let package_dir = current_dir.join("../../packages").join(pkg_name_in_path);
        let temp_dir = package_dir.join("npm");
        if let Ok(()) = fs::remove_dir_all(&temp_dir) {};
        fs::create_dir(&temp_dir).expect("Unable to create temporary npm directory");
        for platform in Vec::<NpmSupportedPlatform>::new().iter() {
            let bin_or_napi_file_name = match pkg.kind {
                NpmPackageKind::Bin(bin) => {
                    if platform.os == "win32" {
                        format!("{}.exe", bin)
                    } else {
                        bin.to_string()
                    }
                }
                NpmPackageKind::Napi => {
                    format!("lib{}.dylib", pkg.crate_name.replace("-", "_"))
                }
            };
            let platform_package_name = format!("{}-{}-{}", pkg.name, platform.os, platform.arch);
            optional_dependencies.push(platform_package_name.clone());
            let mut pkg_json = serde_json::json!({
              "name": platform_package_name,
              "version": version,
              "description": pkg.description,
              "os": [platform.os],
              "cpu": [platform.arch],
            });

            match pkg.kind {
                NpmPackageKind::Bin(bin) => {
                    pkg_json[bin] = bin_or_napi_file_name.clone().into();
                }
                NpmPackageKind::Napi => {
                    pkg_json["main"] = "index.js".into();
                    pkg_json["types"] = "index.d.ts".into();
                }
            };

            let dir_name = format!("{}-{}-{}", pkg_name_in_path, platform.os, platform.arch);
            let target_dir = package_dir.join("npm").join(dir_name);
            fs::create_dir(&target_dir).unwrap_or_else(|e| {
                panic!(
                    "Unable to create dir:
        {:?}\n{e}",
                    &target_dir
                )
            });
            fs::write(
                target_dir.join("package.json"),
                serde_json::to_string_pretty(&pkg_json).unwrap(),
            )
            .expect("Unable to write package.json");
            let artifact_path = current_dir
                .join("artifacts")
                .join(format!("{}-{}", pkg.crate_name, platform.rust_target))
                .join(&bin_or_napi_file_name);
            let dist_path = target_dir.join(&bin_or_napi_file_name);
            fs::copy(&artifact_path, &dist_path).unwrap_or_else(|e| {
                panic!(
                    "Copy file from [{:?}] to [{:?}] failed: {e}",
                    artifact_path, dist_path
                )
            });
            Command::program("npm")
                // TODO --provenance
                .args([
                    "publish",
                    "--provenance",
                    "--access",
                    "public",
                    "--tag",
                    tag,
                ])
                .error_message("Publish npm package failed")
                .current_dir(target_dir)
                .dry_run(dry_run)
                .execute();
>>>>>>> 9f27a5b (feat(xtask): ci utils (#5)):xtask/src/publish/mod.rs
        }

        let target_pkg_dir = temp_dir.join(pkg_name_in_path);
        fs::create_dir_all(&target_pkg_dir).unwrap_or_else(|e| {
            panic!(
                "Unable to create target npm directory [{:?}]: {e}",
                target_pkg_dir
            )
        });
        let optional_dependencies_with_version = optional_dependencies
            .into_iter()
            .map(|name| (name, version.clone()))
            .collect::<HashMap<String, String>>();
        let pkg_json_content =
            fs::read(package_dir.join("package.json")).expect("Unable to read package.json");
        let mut pkg_json: Value = serde_json::from_slice(&pkg_json_content).unwrap();
        pkg_json["optionalDependencies"] =
            serde_json::to_value(optional_dependencies_with_version).unwrap();
        fs::write(
            target_pkg_dir.join("package.json"),
            serde_json::to_string_pretty(&pkg_json).unwrap(),
        )
        .unwrap_or_else(|e| {
            panic!(
                "Write [{:?}] failed: {e}",
                target_pkg_dir.join("package.json")
            )
        });
<<<<<<< HEAD:xtask/src/publish.rs
        // TODO(kijv) windows helper (for .exe)
        // TODO(kijv) gen napi js bindings helper
=======

        let cm = Arc::<SourceMap>::default();
        let c = SwcCompiler::new(cm.clone());

        let minify_js = |src: &str| -> String {
            let output = GLOBALS
                .set(&Default::default(), || {
                    try_with_handler(cm.clone(), Default::default(), |handler| {
                        let fm = cm.new_source_file(FileName::Anon.into(), src.to_string());

                        Ok(c.minify(
                            fm,
                            handler,
                            &JsMinifyOptions {
                                compress: BoolOrDataConfig::from_bool(true),
                                mangle: BoolOrDataConfig::from_bool(true),
                                ..Default::default()
                            },
                        )
                        .expect("failed to minify"))
                    })
                })
                .unwrap();

            output.code
        };

        match pkg.kind {
            NpmPackageKind::Bin(bin) => {
                fs::write(
                    target_pkg_dir.join(bin),
                    minify_js(include_str!("./bin.js")),
                )
                .expect("Unable to write bin helper");
            }
            NpmPackageKind::Napi => {
                let intermediate_type_file = current_dir
                    .join("artifacts")
                    .join(pkg.crate_name)
                    .join(format!("lib{}.typedef", pkg.crate_name.replace("-", "_")));
                let (dts, exports) = process_typedef(intermediate_type_file, false, None)
                    .expect("unable to process typedef");

                fs::write(target_pkg_dir.join("index.d.ts"), dts)
                    .expect("Unable to write index.d.ts");

                if !exports.is_empty() {
                    let cjs =
                        create_cjs_binding(&pkg.crate_name.replace("-", "_"), pkg.name, &exports);

                    fs::write(target_pkg_dir.join("index.js"), minify_js(&cjs))
                        .expect("Unable to write index.js");
                }
            }
        };

>>>>>>> 9f27a5b (feat(xtask): ci utils (#5)):xtask/src/publish/mod.rs
        Command::program("npm")
            .args(["publish", "--access", "public", "--tag", tag])
            .error_message("Publish npm package failed")
            .current_dir(target_pkg_dir)
            .dry_run(dry_run)
            .execute();
    }
}

<<<<<<< HEAD:xtask/src/publish.rs
const VERSION_TYPE: &[&str] = &[
    "patch", "minor", "major", "alpha", "beta", "canary", "nightly",
];
=======
// nightly is just a temporary state when publishing, it's not a real release
// that would contribute to semver
const VERSION_TYPE: &[&str] = &["patch", "minor", "major", "alpha", "beta", "canary"];
>>>>>>> 9f27a5b (feat(xtask): ci utils (#5)):xtask/src/publish/mod.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkspaceProjectMeta {
    #[serde(default = "default_empty_string")]
    name: String,
    path: String,
    private: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PackageJson {
    #[serde(default = "default_empty_string")]
    version: String,
    #[serde(default = "default_empty_string")]
    name: String,
    #[serde(default)]
    private: bool,
    alias: Option<String>,
    #[serde(default = "default_empty_string")]
    path: String,
}

pub fn run_bump(names: HashSet<String>, version_type: Option<&String>, dry_run: bool) {
    let workspaces_list_text = Command::program("pnpm")
        .args(["ls", "-r", "--depth", "-1", "--json"])
        .error_message("List workspaces failed")
        .output_string();
    let workspaces = serde_json::from_str::<Vec<WorkspaceProjectMeta>>(workspaces_list_text.trim())
        .expect("Unable to parse workspaces list")
        .iter()
        .filter_map(|workspace| {
            let workspace_pkg_json = fs::read_to_string(
                env::current_dir()
                    .unwrap()
                    .join(&workspace.path)
                    .join("package.json"),
            )
            .expect("Read workspace package.json failed");
            let mut pkg_json: PackageJson = serde_json::from_str(&workspace_pkg_json)
                .expect("Parse workspace package.json failed");
            if workspace.name.is_empty() || pkg_json.private {
                None
            } else {
                pkg_json.path.clone_from(&workspace.path);
                Some(pkg_json)
            }
        })
        .collect::<Vec<PackageJson>>();
    let mut workspaces_to_bump = workspaces
        .iter()
        .filter(|&p| names.contains(&p.name))
        .cloned()
        .collect::<Vec<_>>();
    if workspaces_to_bump.is_empty() {
        fn name_to_title(package: &PackageJson) -> String {
            format!(
                "{}, current version is {}",
                package.name.bright_cyan(),
                package.version.bright_green()
            )
        }
        let selector = inquire::MultiSelect::new(
            "Select a package to bump",
            workspaces.iter().map(name_to_title).collect(),
        );
        workspaces_to_bump = selector
            .prompt()
            .expect("Failed to prompt packages")
            .iter()
            .filter_map(|p| workspaces.iter().find(|w| name_to_title(w) == *p))
            .cloned()
            .collect();
    }
    let mut tags_to_apply = Vec::new();
    workspaces_to_bump.iter().for_each(|p| {
        let version_type = if let Some(version_type) = version_type {
            version_type.as_str()
        } else {
            let title = format!("Version for {}", &p.name);
            let selector = inquire::Select::new(title.as_str(), VERSION_TYPE.to_owned());
            selector.prompt().expect("Get version type failed")
        };
        let mut semver_version = Version::parse(&p.version).unwrap_or_else(|e| {
            panic!("Failed to parse {} in {} as semver: {e}", p.version, p.name)
        });

        match version_type {
            "major" => {
                semver_version.major += 1;
                semver_version.minor = 0;
                semver_version.patch = 0;
                semver_version.pre = Prerelease::EMPTY;
            }
            "minor" => {
                semver_version.minor += 1;
                semver_version.patch = 0;
                semver_version.pre = Prerelease::EMPTY;
            }
            "patch" => {
                semver_version.patch += 1;
                semver_version.pre = Prerelease::EMPTY;
            }
            "alpha" | "beta" | "canary" => {
                if semver_version.pre.is_empty() {
                    semver_version.patch += 1;
                    semver_version.pre =
                        Prerelease::new(format!("{}.0", version_type).as_str()).unwrap();
                } else {
                    let mut prerelease_version = semver_version.pre.split('.');
                    let prerelease_type = prerelease_version
                        .next()
                        .expect("prerelease type should exist");
                    let prerelease_version = prerelease_version
                        .next()
                        .expect("prerelease version number should exist");
                    let mut version_number = prerelease_version
                        .parse::<u32>()
                        .expect("prerelease version number should be u32");
                    if semver_version.pre.contains(version_type) {
                        version_number += 1;
                        semver_version.pre = Prerelease::new(
                            format!("{}.{}", version_type, version_number).as_str(),
                        )
                        .unwrap();
                    } else {
                        // eg. current version is 1.0.0-beta.12, bump to 1.0.0-canary.0
                        if Prerelease::from_str(version_type).unwrap()
                            > Prerelease::from_str(prerelease_type).unwrap()
                        {
                            semver_version.pre =
                                Prerelease::new(format!("{}.0", version_type).as_str()).unwrap();
                        } else {
                            panic!(
                                "Previous version is {prerelease_type}, so you can't bump to \
                                 {version_type}",
                            );
                        }
                    }
                }
            }
            _ => unreachable!(),
        }
        let semver_version_string = semver_version.to_string();
        let version_command_args = vec![
            "version",
            semver_version_string.as_str(),
            "--no-git-tag-version",
            "--no-commit-hooks",
        ];
        Command::program("pnpm")
            .args(version_command_args)
            .current_dir(PathBuf::from(&p.path))
            .dry_run(dry_run)
            .error_message("Bump version failed")
            .execute();
        tags_to_apply.push(format!(
            "{}@{}",
            p.alias.as_ref().unwrap_or(&p.name),
            semver_version_string
        ));
    });
    Command::program("pnpm")
        .args(["install"])
        .dry_run(dry_run)
        .error_message("Update pnpm-lock.yaml failed")
        .execute();
    Command::program("git")
        .args(["add", "."])
        .dry_run(dry_run)
        .error_message("Stash git changes failed")
        .execute();
    let tags_message = tags_to_apply
        .iter()
        .map(|s| format!("- {s}"))
        .collect::<Vec<_>>()
        .join("\n");
    Command::program("git")
        .args([
            "commit",
            "-m",
            format!(
                "chore: release npm package{}",
                if tags_to_apply.len() > 1 { "s" } else { "" }
            )
            .as_str(),
            "-m",
            tags_message.as_str(),
        ])
        .dry_run(dry_run)
        .error_message("Stash git changes failed")
        .execute();
    for tag in tags_to_apply {
        Command::program("git")
            .dry_run(dry_run)
            .args(["tag", "-s", &tag, "-m", &tag])
            .error_message("Tag failed")
            .execute();
    }
}

// TODO do we even keep this?
#[allow(dead_code)]
pub fn publish_workspace(is_nightly: bool, dry_run: bool) {
    let commit_message = Command::program("git")
        .args(["log", "-1", "--pretty=%B"])
        .error_message("Get commit hash failed")
        .output_string();
    for (pkg_name_without_scope, scope, version) in commit_message
        .trim()
        .split('\n')
        // Skip commit title
        .skip(1)
        .map(|s| s.trim().trim_start_matches('-').trim())
        .map(|m| {
            let scope = if m.starts_with("@fujinoki/") {
                Some("@fujinoki/")
            } else {
                None
            };
            let m = m.trim_start_matches("@fujinoki/");
            let mut full_tag = m.split('@');
            let pkg_name_without_scope = full_tag.next().unwrap().to_string();
            let version = if is_nightly {
                get_nightly_version()
            } else {
                full_tag.next().unwrap().to_string()
            };
            (pkg_name_without_scope, scope, version)
        })
    {
        let pkg_name = format!("{}{pkg_name_without_scope}", scope.unwrap_or_default());
        let semver_version = Version::from_str(version.as_str())
            .unwrap_or_else(|e| panic!("Parse semver version failed {version} {e}"));
        let is_alpha = semver_version.pre.contains("alpha");
        let is_beta = semver_version.pre.contains("beta");
        let is_canary = semver_version.pre.contains("canary");
        let tag = {
            if is_nightly {
                "nightly"
            } else if is_alpha {
                "alpha"
            } else if is_beta {
                "beta"
            } else if is_canary {
                "canary"
            } else {
                "latest"
            }
        };
        let mut args = vec![
            "publish",
            "--provenance",
            "--tag",
            tag,
            "--no-git-checks",
            "--filter",
            pkg_name.as_str(),
        ];
        if dry_run {
            args.push("--dry-run");
        }
        Command::program("pnpm")
            .args(args)
            .error_message("Publish failed")
            .execute();
    }
}
