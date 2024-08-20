use std::collections::BTreeMap;

use anyhow::Result;
use tracing::Instrument;
use turbopack_binding::{
    turbo::{
        tasks as turbo_tasks,
        tasks::{Completion, RcStr, ValueToString, Vc},
        tasks_fs::{FileSystemEntryType, FileSystemPath},
    },
    turbopack::core::issue::{IssueExt, IssueSeverity, StyledString},
};

use super::{get_directory_tree, issue::DirectoryTreeIssue, DirectoryTree};
use crate::config::FujinokiConfig;

#[turbo_tasks::value(shared)]
#[derive(Clone, Debug)]
pub struct CommandMetadata {
    pub name: RcStr,
    pub file_path: Vc<FileSystemPath>,
}

#[turbo_tasks::value(shared, transparent)]
#[derive(Clone, Debug, Default)]
pub struct CommandsMetadata(pub Vec<CommandMetadata>);

#[turbo_tasks::value(transparent)]
#[derive(Default)]
pub struct OptionCommandsDir(Option<Vc<FileSystemPath>>);

#[turbo_tasks::value_impl]
impl OptionCommandsDir {
    /// Returns a completion that changes when any route in the whole tree
    /// changes.
    #[turbo_tasks::function]
    pub async fn routes_changed(
        self: Vc<Self>,
        config: Vc<FujinokiConfig>,
    ) -> Result<Vc<Completion>> {
        if let Some(dir) = *self.await? {
            let directory_tree = get_directory_tree(dir, config.file_extensions());
            directory_tree.routes_changed().await?;
        }
        Ok(Completion::new())
    }
}

/// Finds and returns the [DirectoryTree] of the _ directory if existing.
#[turbo_tasks::function]
pub async fn find_commands_dir(project_path: Vc<FileSystemPath>) -> Result<Vc<OptionCommandsDir>> {
    let app = project_path.join("commands".to_string().into());
    let src_app = project_path.join("src/commands".to_string().into());
    let dir = if *app.get_type().await? == FileSystemEntryType::Directory {
        app
    } else if *src_app.get_type().await? == FileSystemEntryType::Directory {
        src_app
    } else {
        return Ok(Vc::cell(None));
    }
    .resolve()
    .await?;

    Ok(Vc::cell(Some(dir)))
}

#[turbo_tasks::function]
pub async fn directory_tree_to_commands_metadata(
    dir: Vc<FileSystemPath>,
    directory_tree: Vc<DirectoryTree>,
) -> Result<Vc<CommandsMetadata>> {
    let span = {
        let dir = dir.to_string().await?.to_string();
        tracing::info_span!("commands directory tree into data", name = dir)
    };
    directory_tree_to_commands_metadata_internal(dir, directory_tree)
        .instrument(span)
        .await
}

async fn directory_tree_to_commands_metadata_internal(
    dir: Vc<FileSystemPath>,
    directory_tree: Vc<DirectoryTree>,
) -> Result<Vc<CommandsMetadata>> {
    directory_tree.routes_changed().await?;

    let mut commands: CommandsMetadata = CommandsMetadata::default();

    let DirectoryTree {
        subdirectories,
        components,
    } = &*directory_tree.await?;

    for (name, subdirectory) in subdirectories {
        let sub = directory_tree_to_commands_metadata(dir.clone(), *subdirectory).await?;
        for metadata in sub {
            if is_valid_route(name.clone().as_str()) {
                match metadata.name.as_str() {
                    "command" => {
                        commands.0.push(CommandMetadata {
                            name: name.clone(),
                            file_path: metadata.file_path,
                        });
                    }
                    _ => {
                        if is_group_route(&name) {
                            commands.0.push(CommandMetadata {
                                name: metadata.name.clone(),
                                ..*metadata
                            });
                        }
                    }
                }
            }
        }
    }

    for (name, file) in &components.await?.clone_value().0 {
        if is_valid_route(name.clone().as_str()) {
            commands.0.push(CommandMetadata {
                name: name.clone(),
                file_path: file.clone(),
            });
        } else {
            DirectoryTreeIssue {
                dir,
                message: StyledString::Text(
                    format!(
                        "Invalid command name: {} at {}, ignoring command",
                        name,
                        file.realpath().await?.to_string(),
                    )
                    .into(),
                )
                .cell(),
                severity: IssueSeverity::Warning.cell(),
            }
            .cell()
            .emit();
        }
    }

    let mut seen_conflicts: BTreeMap<RcStr, CommandMetadata> = Default::default();
    for (name, event) in commands.0.iter().enumerate() {
        for (other_name, other_event) in commands.0.iter().enumerate() {
            if name != other_name
                && event.name == other_event.name
                && !seen_conflicts.contains_key(&event.name)
            {
                conflict_issue(
                    dir.clone(),
                    event.clone().name.to_string(),
                    "command",
                    "command",
                    &event.file_path.to_string().await?.to_string(),
                    &other_event.file_path.to_string().await?.to_string(),
                );
                seen_conflicts.insert(event.name.clone(), event.clone());
            }
        }
    }

    Ok(commands.cell())
}

fn is_group_route(name: &str) -> bool {
    name.starts_with('(') && name.ends_with(')')
}

fn is_valid_route(name: &str) -> bool {
    discord_api::application::command::CHAT_INPUT_NAME.is_match(name)
}

fn conflict_issue(
    dir: Vc<FileSystemPath>,
    e: String,
    a: &str,
    b: &str,
    value_a: &String,
    value_b: &String,
) {
    let item_names = if a == b {
        format!("{}s", a)
    } else {
        format!("{} and {}", a, b)
    };

    DirectoryTreeIssue {
        dir,
        message: StyledString::Text(
            format!(
                "Conflicting {} at {}: {a} at {value_a} and {b} at {value_b}",
                item_names, e,
            )
            .into(),
        )
        .cell(),
        severity: IssueSeverity::Fatal.cell(),
    }
    .cell()
    .emit();
}
