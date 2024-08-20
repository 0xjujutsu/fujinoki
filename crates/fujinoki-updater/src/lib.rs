#![deny(clippy::all)]

use std::{collections::HashMap, fmt, time::Duration};

use console::style;
use fujinoki_core::{DISPLAY_NAME, NPM_PACKAGE};
use semver::Version as SemVerVersion;
use serde::Deserialize;
use thiserror::Error as ThisError;
use update_informer::{
    http_client::{GenericHttpClient, HeaderMap, HttpClient},
    Check, Package, Registry, Result as UpdateResult, Version,
};

// 800ms
const DEFAULT_TIMEOUT: Duration = Duration::from_millis(800);
// 1 day
const DEFAULT_INTERVAL: Duration = Duration::from_secs(60 * 60 * 24);

const NOTIFIER_DISABLE_VARS: [&str; 2] = ["NO_UPDATE_NOTIFIER", "FUJINOKI_NO_UPDATE_NOTIFIER"];
const ENVIRONMENTAL_DISABLE_VARS: [&str; 1] = ["CI"];

const REGISTRY: &str = "https://registry.npmjs.org";

#[derive(ThisError, Debug)]
pub enum UpdateNotifierError {
    #[error("Failed to parse current version")]
    VersionError(#[from] semver::Error),
    #[error("Failed to check for updates")]
    FetchError(#[from] Box<dyn std::error::Error>),
}

#[derive(Deserialize, Debug)]
pub struct FetchDistTags {
    #[serde(rename = "dist-tags")]
    pub dist_tags: HashMap<String, String>,
}

#[derive(Debug)]
enum VersionTag {
    Latest,
    Canary,
}

impl fmt::Display for VersionTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VersionTag::Latest => write!(f, "latest"),
            VersionTag::Canary => write!(f, "canary"),
        }
    }
}

struct NPMRegistry;

impl Registry for NPMRegistry {
    const NAME: &'static str = "npm-registry";
    fn get_latest_version<T: HttpClient>(
        http: GenericHttpClient<T>,
        pkg: &Package,
    ) -> UpdateResult<Option<String>> {
        // determine tag to request
        let tag = get_tag_from_version(&pkg.version().semver().pre);
        // since we're overloading tag within name, we need to split it back out
        let full_name = pkg.to_string();
        let split_name: Vec<&str> = full_name.split('/').collect();
        let name = split_name[1];
        let url = make_registry_url(name);
        let result: FetchDistTags = http.get(&url)?;

        Ok(result.dist_tags.get(&tag.to_string()).cloned())
    }
}

// Source https://github.com/mgrachev/update-informer/blob/main/src/http_client/reqwest.rs
// Vendored here until update-informer allows us to control tls implementation
pub struct ReqwestHttpClient;

impl HttpClient for ReqwestHttpClient {
    fn get<T: serde::de::DeserializeOwned>(
        url: &str,
        timeout: Duration,
        headers: HeaderMap,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let mut req = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .build()?
            .get(url);

        for (key, value) in headers {
            req = req.header(key, value);
        }

        let json = req.send()?.json()?;

        Ok(json)
    }
}

fn get_tag_from_version(pre: &semver::Prerelease) -> VersionTag {
    match pre {
        t if t.contains("canary") => VersionTag::Canary,
        _ => VersionTag::Latest,
    }
}

fn should_skip_notification() -> bool {
    NOTIFIER_DISABLE_VARS
        .iter()
        .any(|var| std::env::var(var).is_ok())
        || ENVIRONMENTAL_DISABLE_VARS
            .iter()
            .any(|var| std::env::var(var).is_ok())
        || !atty::is(atty::Stream::Stdout)
}

pub fn display_update_check(
    package_name: &str,
    current_version: &str,
    timeout: Option<Duration>,
    interval: Option<Duration>,
) -> Result<(), UpdateNotifierError> {
    // bail early if the user has disabled update notifications
    if should_skip_notification() {
        return Ok(());
    }

    let version = check_for_updates(package_name, current_version, timeout, interval);

    if let Ok(Some(version)) = version {
        let latest_version = version.to_string();

        let msg = format!(
            "{DISPLAY_NAME} v{latest_version} is out! You're on {current_version}.\nRun \
             {update_cmd} to upgrade.
            ",
            latest_version = style(latest_version).bold().cyan(),
            current_version = style(format!("v{}", current_version)).bold().cyan(),
            update_cmd = style(format!("{NPM_PACKAGE} upgrade").trim().to_string())
                .cyan()
                .bold()
        );

        println!("{}", msg);
    }

    Ok(())
}

pub fn extract_package_name(
    package_name: &str,
    current_version: &str,
) -> Result<String, UpdateNotifierError> {
    // we want notifications per channel (latest, canary, etc) so we need to ensure
    // we have one cached latest version per channel. UpdateInformer does not
    // support this out of the box, so we hack it into the name by overloading
    // owner (in the supported owner/name format) to be channel/name.
    let parsed_version = SemVerVersion::parse(current_version)?;
    let tag = get_tag_from_version(&parsed_version.pre);
    Ok(format!("{}/{}", tag, package_name))
}

pub fn check_for_updates(
    package_name: &str,
    current_version: &str,
    timeout: Option<Duration>,
    interval: Option<Duration>,
) -> Result<Option<Version>, UpdateNotifierError> {
    let package_name = extract_package_name(package_name, current_version)?;

    let timeout = timeout.unwrap_or(DEFAULT_TIMEOUT);
    let interval = interval.unwrap_or(DEFAULT_INTERVAL);
    let informer = update_informer::new(NPMRegistry, package_name.clone(), current_version)
        .http_client(ReqwestHttpClient)
        .timeout(timeout)
        .interval(interval);
    let data = informer
        .check_version()
        .map_err(UpdateNotifierError::FetchError)?;

    Ok(data)
}

pub fn make_registry_url(name: &str) -> String {
    format!("{}/{}", REGISTRY, name)
}

#[cfg(all(feature = "native-tls", feature = "rustls-tls"))]
compile_error!("You can't enable both `native-tls` and `rustls-tls`");

#[cfg(all(not(feature = "native-tls"), not(feature = "rustls-tls")))]
compile_error!("You have to enable one of the TLS backends: `native-tls` or `rustls-tls`");
