use anyhow::{Context, Result};
use discord_api::id::ApplicationId;
use fujinoki_core::{
    config::FujinokiConfig,
    structures::commands::{CommandMetadata, CommandsMetadata},
};
use fujinoki_node::transforms::exports::ExportsContext;
use serde_json::{json, Value as JsonValue};
use turbopack_binding::{
    turbo::{
        tasks::{self as turbo_tasks, Completion, ValueToString, Vc},
        tasks_bytes::stream::SingleValue,
        tasks_fs::json::parse_json_with_source_context,
    },
    turbopack::{
        core::{
            file_source::FileSource,
            issue::{handle_issues, IssueReporter, IssueSeverity},
            module::Module,
            source::Source,
        },
        node::evaluate::custom_evaluate,
    },
};

use super::util::merge_json;
use crate::source::{ContentSourceData, EntryMap};

#[turbo_tasks::value]
#[derive(Clone, Debug)]
pub struct DiscordApplicationCommandsUpdater {
    config: Vc<FujinokiConfig>,
    resolved_source: Vc<ContentSourceData>,
    issue_reporter: Vc<Box<dyn IssueReporter>>,
    application_id: ApplicationId,
}

#[turbo_tasks::value_impl]
impl DiscordApplicationCommandsUpdater {
    #[turbo_tasks::function]
    pub async fn update_application_commands(
        self: Vc<Self>,
        commands: Vc<CommandsMetadata>,
        entries: Vc<EntryMap>,
        additional_invalidation: Vc<Option<Vc<Completion>>>,
    ) -> Result<Vc<()>> {
        if let Some(additional_invalidation) = *additional_invalidation.await? {
            additional_invalidation.await?;
        };

        let config = self.await?.config;
        let resolved_source = self.await?.resolved_source;
        let issue_reporter = self.await?.issue_reporter;
        let application_id = self.await?.application_id;

        let existing_application_commands =
            discord_api::rest::application::commands::get_global_application_commands(
                application_id,
                false,
                config.client().token(),
                resolved_source.await?.commands_dir.await?.clone_value(),
            );

        handle_issues(
            existing_application_commands.clone(),
            issue_reporter,
            IssueSeverity::Fatal.cell(),
            None,
            Some("get global application commands"),
        )
        .await?;

        let existing_application_commands = (&*existing_application_commands.await?)
            .iter()
            .map(|value| *value)
            .collect::<Vec<_>>();

        for command_data in commands.await?.iter() {
            handle_issues(
                self.update_application_command(
                    command_data.clone().cell(),
                    entries.get_entry(command_data.file_path.to_string()),
                    additional_invalidation,
                ),
                issue_reporter,
                IssueSeverity::Fatal.cell(),
                None,
                Some("update application command"),
            )
            .await?;
        }

        for application_command in existing_application_commands.iter() {
            let application_command = application_command.await?;

            if let None = commands
                .await?
                .iter()
                .find(|v| v.name == application_command.name)
            {
                let res =
                    discord_api::rest::application::command::delete_global_application_command(
                        application_id,
                        application_command.id,
                        config.client().token(),
                        resolved_source.await?.commands_dir.await?.clone_value(),
                    );
                handle_issues(res, issue_reporter, IssueSeverity::Fatal.cell(), None, None).await?;
            }
        }

        Ok(Default::default())
    }

    #[turbo_tasks::function]
    async fn update_application_command(
        self: Vc<Self>,
        command_data: Vc<CommandMetadata>,
        entry: Vc<Option<Vc<Box<dyn Module>>>>,
        additional_invalidation: Vc<Option<Vc<Completion>>>,
    ) -> Result<Vc<()>> {
        if let Some(additional_invalidation) = *additional_invalidation.await? {
            additional_invalidation.await?;
        };

        let config = self.await?.config;
        let resolved_source = self.await?.resolved_source;
        let issue_reporter = self.await?.issue_reporter;
        let application_id = self.await?.application_id;

        // This should be okay because the result should already be cached.
        let registered_application_commands =
            discord_api::rest::application::commands::get_global_application_commands(
                application_id,
                false,
                config.client().token(),
                resolved_source.await?.commands_dir.await?.clone_value(),
            );

        handle_issues(
            registered_application_commands.clone(),
            issue_reporter,
            IssueSeverity::Fatal.cell(),
            None,
            None,
        )
        .await?;

        let mut existing_application_commands = vec![];
        for application_command in registered_application_commands.await?.iter() {
            existing_application_commands.push(application_command.await?.clone());
        }

        let command_data = command_data.await?;
        let exported: JsonValue = if let Some(entry) = *entry.await? {
            let project_path = resolved_source.clone().await?.project_path;
            let chunking_context = resolved_source
                .clone()
                .await?
                .executor
                .await?
                .chunking_context;
            let asset_context = resolved_source.clone().await?.executor.await?.asset_context;
            let env = resolved_source.clone().await?.executor.await?.env;

            let initial_val = custom_evaluate(ExportsContext {
                // TODO(kijv) use individual exported values AND the data object which contains
                // EVERY key (to be merged with individual exports having precedence)
                args: vec![Vc::cell("data".into())],
                module_asset: entry,
                cwd: project_path,
                env,
                context_ident_for_issue: FileSource::new(project_path).ident(),
                asset_context,
                chunking_context: Vc::upcast(chunking_context),
                resolve_options_context: None,
                additional_invalidation: resolved_source.await?.commands_dir.routes_changed(config),
            });
            let SingleValue::Single(val) = initial_val.await?.try_into_single().await? else {
                // An error happened, which has already been converted into an issue.
                handle_issues(
                    initial_val,
                    issue_reporter,
                    IssueSeverity::Fatal.cell(),
                    None,
                    None,
                )
                .await?;
                return Ok(Default::default());
            };
            parse_json_with_source_context(val.to_str()?)
                .context("Unable to deserialize response")?
        } else {
            return Ok(Default::default());
        };
        dbg!(exported.clone());
        let data = exported.get("data").unwrap();

        // edit
        if let Some(application_command) = existing_application_commands
            .iter()
            .find(|v| v.name.eq(&command_data.name.to_string()))
        {
            let mut command = json!({
                "name": command_data.name,
                // TODO(kijv) make `description`/`data.description` a required export
                "description": "Hello world!"
            });
            merge_json(&mut command, &data);

            let res = discord_api::rest::application::command::edit_global_application_command(
                application_id,
                application_command.id,
                Vc::cell(command.clone()),
                config.client().token(),
                Some(command_data.file_path),
            );
            handle_issues(res, issue_reporter, IssueSeverity::Fatal.cell(), None, None).await?;
        }
        // create
        else {
            let mut command = json!({
                "name": command_data.name,
            });
            merge_json(&mut command, &data);

            let res = discord_api::rest::application::command::create_global_application_command(
                application_id,
                Vc::cell(command.clone()),
                config.client().token(),
                Some(command_data.file_path),
            );
            handle_issues(res, issue_reporter, IssueSeverity::Fatal.cell(), None, None).await?;
        }

        Ok(Default::default())
    }
}
