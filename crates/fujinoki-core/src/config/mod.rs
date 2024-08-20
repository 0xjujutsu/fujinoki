use std::sync::{Arc, OnceLock};

// TODO use biome_deserialize
use anyhow::{Context, Ok, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use turbopack_binding::{
    turbo::{
        tasks as turbo_tasks,
        tasks::{RcStr, ValueToString, Vc},
        tasks_env::EnvMap,
        tasks_fs::{FileJsonContent, FileSystemPath},
    },
    turbopack::core::issue::{IssueExt, StyledString},
};

use self::issue::ConfigIssue;

pub mod issue;

static CONFIG_FILE: OnceLock<Arc<Vc<FileSystemPath>>> = OnceLock::new();

#[turbo_tasks::value(shared, serialization = "custom", eq = "manual")]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TokenOptionsOrString {
    String(RcStr),
    /// Map of the current environment to the token.
    Options(IndexMap<String, String>),
}

#[turbo_tasks::value(shared, serialization = "custom", eq = "manual")]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    token: Option<TokenOptionsOrString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    intents: Option<u32>,
}

#[turbo_tasks::value]
pub struct OptionalClientOptions {
    pub(crate) inner: Option<Vc<ClientOptions>>,
    pub(crate) node_env: RcStr,
}

#[turbo_tasks::value_impl]
impl OptionalClientOptions {
    #[turbo_tasks::function]
    pub async fn token(self: Vc<Self>) -> Result<Vc<RcStr>> {
        let options = if let Some(options) = self.await?.inner {
            &options.await?.token
        } else {
            &None
        };

        let token = match options {
            Some(TokenOptionsOrString::Options(map)) => {
                let token = map
                    .get(self.await?.node_env.as_str())
                    .context("Token not found")?;
                token.clone().into()
            }
            Some(TokenOptionsOrString::String(token)) => token.clone().into(),
            None => {
                ConfigIssue {
                    path: *CONFIG_FILE.get().unwrap().clone(),
                    description: StyledString::Text(
                        "Missing `client.token`, this is required in order to properly run your \
                         application"
                            .into(),
                    )
                    .cell(),
                }
                .cell()
                .emit();

                return Ok(Vc::cell(Default::default()));
            }
        };

        Ok(Vc::cell(token))
    }

    #[turbo_tasks::function]
    pub async fn intents(self: Vc<Self>) -> Result<Vc<u32>> {
        if let Some(options) = self.await?.inner {
            if let Some(intents) = options.await?.intents {
                return Ok(Vc::cell(intents));
            }
        }

        Ok(Vc::cell(0))
    }
}

#[turbo_tasks::value(serialization = "custom", eq = "manual")]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FujinokiConfig {
    pub client: Option<ClientOptions>,
    pub env: Option<IndexMap<String, JsonValue>>,
    pub file_extensions: Option<Vec<RcStr>>,
    #[serde(skip)]
    pub(crate) node_env: RcStr,
}

impl FujinokiConfig {
    pub fn node_env(mut self, node_env: RcStr) -> Self {
        self.node_env = node_env;
        self
    }
}

#[turbo_tasks::value_impl]
impl FujinokiConfig {
    #[turbo_tasks::function]
    pub async fn from_string(string: Vc<RcStr>, node_env: Option<RcStr>) -> Result<Vc<Self>> {
        let string = string.await?;
        let config: FujinokiConfig = serde_json::from_str(&string)
            .with_context(|| format!("failed to parse config: {}", string))?;
        Ok(config
            .node_env(node_env.expect("node_env is required"))
            .cell())
    }

    #[turbo_tasks::function]
    pub async fn from_json(
        json_path: Vc<FileSystemPath>,
        node_env: Option<RcStr>,
    ) -> Result<Vc<Self>> {
        let json = json_path.read_json();
        let json = if let FileJsonContent::NotFound = *json_path.read_json().await? {
            ConfigIssue {
                path: json_path,
                description: StyledString::Text("Config file not found".into()).cell(),
            }
            .cell()
            .emit();

            return Ok(FujinokiConfig::default().cell());
        } else {
            json
        };

        let config = FujinokiConfig::from_string(json.to_string(), node_env);

        CONFIG_FILE.set(Arc::new(json_path)).unwrap();

        Ok(config)
    }

    // TODO Vite style config loading
    #[turbo_tasks::function]
    pub async fn from_js() {
        todo!()
    }

    #[turbo_tasks::function]
    pub async fn client(self: Vc<Self>) -> Result<Vc<OptionalClientOptions>> {
        let options = &self.await?.client;

        let options = OptionalClientOptions {
            inner: options.clone().map(|o| o.cell()),
            node_env: "development".into(),
        };

        Ok(options.cell())
    }

    #[turbo_tasks::function]
    pub async fn env(self: Vc<Self>) -> Result<Vc<EnvMap>> {
        // The value expected for env is Record<String, String>, but config itself
        // allows arbitary object (https://github.com/vercel/next.js/blob/25ba8a74b7544dfb6b30d1b67c47b9cb5360cb4e/packages/next/src/server/config-schema.ts#L203)
        // then stringifies it. We do the interop here as well.
        let env = self
            .await?
            .env
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|(k, v)| {
                (
                    k.as_str().into(),
                    if let JsonValue::String(s) = v {
                        // A string value is kept, calling `to_string` would wrap in to quotes.
                        s.as_str().into()
                    } else {
                        v.to_string().into()
                    },
                )
            })
            .collect();

        Ok(Vc::cell(env))
    }

    #[turbo_tasks::function]
    pub async fn file_extensions(self: Vc<Self>) -> Result<Vc<Vec<RcStr>>> {
        Ok(Vc::cell(
            self.await?.file_extensions.clone().unwrap_or(
                vec!["js", "jsx", "ts", "tsx"]
                    .iter()
                    .map(|s| s.to_string().into())
                    .collect::<_>(),
            ),
        ))
    }
}
