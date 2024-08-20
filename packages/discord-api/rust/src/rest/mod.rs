use std::{str::FromStr, sync::Arc};

use discord_api::{id::InteractionId, interactions::InteractionResponse};
use turbopack_binding::turbo::{
    tasks::{TurboTasks, Vc},
    tasks_memory::MemoryBackend,
};

use self::interactions::NapiCreateInteractionResponseOptions;
use crate::register;

mod interactions;

#[napi(js_name = "REST")]
pub struct NapiREST {
    turbo_tasks: Arc<TurboTasks<MemoryBackend>>,
}

#[napi(object)]
pub struct NapiTurboEngineOptions {
    /// An upper bound of memory that the Turbo engine will attempt to stay
    /// under.
    pub memory_limit: Option<f64>,
}

#[napi]
impl NapiREST {
    #[napi(constructor)]
    pub fn new(turbo_engine_options: Option<NapiTurboEngineOptions>) -> Self {
        register();

        let turbo_tasks_memory_limit = turbo_engine_options
            .unwrap_or(NapiTurboEngineOptions { memory_limit: None })
            .memory_limit
            .map(|m| m as usize)
            .unwrap_or(usize::MAX);
        let turbo_tasks = TurboTasks::new(MemoryBackend::new(turbo_tasks_memory_limit));

        NapiREST { turbo_tasks }
    }

    #[napi]
    pub async fn create_interaction_response(
        &self,
        interaction_response: serde_json::Value,
        options: NapiCreateInteractionResponseOptions,
    ) -> napi::Result<()> {
        let interaction_response: InteractionResponse =
            serde_json::from_value(interaction_response)?;

        self.turbo_tasks
            .run_once(Box::pin(async move {
                discord_api::rest::interactions::create_interaction_response(
                    InteractionId::from_str(options.interaction_id.as_str())
                        .unwrap()
                        .cell(),
                    Vc::cell(options.interaction_token.into()),
                    interaction_response.cell(),
                    None,
                )
                .await
            }))
            .await
            .unwrap();

        Ok(())
    }
}
