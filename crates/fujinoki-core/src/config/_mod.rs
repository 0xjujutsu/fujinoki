// TODO Also allow usage with .js and .ts (dynamic and/or type-safety)
use anyhow::{bail, Result};
use lazy_regex::{lazy_regex, Lazy};
use regex::Regex;
use serde::{Deserialize, Serialize};
use toml::Value as TomlValue;
use turbopack_binding::turbo::tasks::{trace::TraceRawVcs, Completion, RcStr, Vc};
use turbopack_binding::{
    turbo::tasks_fs::{FileContent, FileSystem, FileSystemPath},
    turbopack::core::issue::{IssueExt, StyledString},
};

use crate::{config::issue::ConfigIssue, NPM_PACKAGE};

pub mod issue;

/// ex. env(DISCORD_TOKEN)
static ENV_FN_PATTERN: Lazy<Regex> = lazy_regex!(r"env\((?P<env_var_name>[A-Z_]+)\)");

#[turbo_tasks::value(serialization = "custom", eq = "manual")]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FujinokiPartialConfig {
    pub client: Option<PartialClientOptions>,
    pub file_extensions: Option<Vec<RcStr>>,
    invalidation: Vc<Completion>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TraceRawVcs)]
pub struct PartialClientOptions {
    pub token: Option<RcStr>,
    pub intents: Option<u32>,
}

#[turbo_tasks::value(serialization = "custom", eq = "manual")]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FujinokiConfig {
    client: ClientOptions,
    file_extensions: Vec<RcStr>,
    invalidation: Vc<Completion>,
}

#[turbo_tasks::value(serialization = "custom", eq = "manual")]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClientOptions {
    // TODO token = { development = "...", production = "..." }
    pub token: RcStr,
    pub intents: u32,
}

#[turbo_tasks::value_impl]
impl FujinokiPartialConfig {
    #[turbo_tasks::function]
    pub async fn load(project_path: Vc<FileSystemPath>) -> Result<Vc<FujinokiPartialConfig>> {
        let config_file_name: RcStr = format!("{NPM_PACKAGE}.toml").into();
        let config_path = project_path.join(config_file_name.clone());
        let content = config_path.read().await?;

        let FileContent::Content(file) = &*content else {
            ConfigIssue {
                path: config_path,
                description: StyledString::Text(
                    format!("Could not find `{config_file_name}`").into(),
                )
                .cell(),
            }
            .cell()
            .emit();

            return Ok(FujinokiPartialConfig {
                client: None,
                file_extensions: None,
                invalidation: Completion::immutable(),
            }
            .cell());
        };

        let toml = &*file.content().to_str()?.to_string();
        let val = toml.parse::<TomlValue>()?;
        let client = val.get("client").map(|v| {
            let token = v
                .get("token")
                .map(|v| v.as_str().unwrap().to_string().into());
            let intents = v.get("intents").map(|v| v.as_integer().unwrap() as u32);
            PartialClientOptions { token, intents }
        });
        let file_extensions = val.get("file_extensions").map(|v| {
            v.as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_string().into())
                .collect::<Vec<RcStr>>()
        });
        Ok(FujinokiPartialConfig {
            client,
            file_extensions,
            invalidation: project_path.fs().track(config_path),
        }
        .cell())
    }
}

impl ClientOptions {
    pub async fn validate(client: Option<PartialClientOptions>) -> Result<Self> {
        if let Some(client) = client {
            let token = match &client.token {
                Some(token) => {
                    if token.is_empty() {
                        // TODO use Issue instead of bail
                        bail!("Token (`client.token`) is empty. Please add a token to continue.");
                    } else if ENV_FN_PATTERN.is_match(token) {
                        let env_var_name = ENV_FN_PATTERN
                            .captures(token)
                            .unwrap()
                            .name("env_var_name")
                            .unwrap()
                            .as_str();
                        let env_var_value = std::env::var(env_var_name).unwrap_or_default();

                        if env_var_value.is_empty() {
                            bail!(
                                "Environment variable `{}` (client.token) is empty. Please set it \
                                 to continue.",
                                env_var_name
                            );
                        }

                        env_var_value.into()
                    } else {
                        token.clone()
                    }
                }
                None => {
                    bail!("Missing `client.token`. Please add a token to continue.");
                }
            };

            // client.intents
            let intents = match client.intents {
                Some(intents) => intents,
                None => 0,
            };

            Ok(Self { token, intents })
        } else {
            bail!("Missing `client` table. Please add it to continue.");
        }
    }

    #[turbo_tasks::function]
    pub async fn token(self: Vc<Self>) -> Result<Vc<RcStr>> {
        Ok(self.await?.token.clone())
    }

    #[turbo_tasks::function]
    pub async fn intents(self: Vc<Self>) -> Result<Vc<u32>> {
        invalidation.await?;
        Ok(self.await?.intents.clone())
    }
}

#[turbo_tasks::value_impl]
impl FujinokiConfig {
    #[turbo_tasks::function]
    pub async fn validate(toml: Vc<FujinokiPartialConfig>) -> Result<Vc<Self>> {
        let toml = toml.await.expect("configuration");
        let client = ClientOptions::validate(toml.client.clone()).await?;
        let file_extensions = toml.file_extensions.clone().unwrap_or(
            vec!["js", "jsx", "ts", "tsx"]
                .iter()
                .map(|s| s.to_string().into())
                .collect::<Vec<RcStr>>(),
        );

        Ok(FujinokiConfig {
            client,
            file_extensions,
            invalidation: toml.invalidation.clone(),
        }
        .cell())
    }

    #[turbo_tasks::function]
    pub async fn client(self: Vc<Self>) -> Result<Vc<ClientOptions>> {
        self.await?.invalidation.await?;
        Ok(self.await?.client.clone().cell())
    }

    #[turbo_tasks::function]
    pub async fn file_extensions(self: Vc<Self>) -> Result<Vc<Vec<RcStr>>> {
        self.await?.invalidation.await?;
        Ok(Vc::cell(self.await?.file_extensions.clone()))
    }
}
