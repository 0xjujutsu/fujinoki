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

#[turbo_tasks::value]
#[derive(Clone, Debug)]
pub struct EventMetadata {
    pub name: RcStr,
    pub file_path: Vc<FileSystemPath>,
}

#[turbo_tasks::value(transparent)]
#[derive(Clone, Debug, Default)]
pub struct EventsMetadata(pub Vec<EventMetadata>);

#[turbo_tasks::value(transparent)]
#[derive(Default)]
pub struct OptionEventsDir(Option<Vc<FileSystemPath>>);

#[turbo_tasks::value_impl]
impl OptionEventsDir {
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
pub async fn find_events_dir(project_path: Vc<FileSystemPath>) -> Result<Vc<OptionEventsDir>> {
    let app = project_path.join("events".to_string().into());
    let src_app = project_path.join("src/events".to_string().into());
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
pub async fn directory_tree_to_events_metadata(
    dir: Vc<FileSystemPath>,
    directory_tree: Vc<DirectoryTree>,
) -> Result<Vc<EventsMetadata>> {
    let span = {
        let dir = dir.to_string().await?.to_string();
        tracing::info_span!("events directory tree into data", name = dir)
    };
    directory_tree_to_events_metadata_internal(dir, directory_tree)
        .instrument(span)
        .await
}

async fn directory_tree_to_events_metadata_internal(
    dir: Vc<FileSystemPath>,
    directory_tree: Vc<DirectoryTree>,
) -> Result<Vc<EventsMetadata>> {
    directory_tree.routes_changed().await?;

    let mut events: EventsMetadata = EventsMetadata::default();

    let DirectoryTree {
        subdirectories,
        components,
    } = &*directory_tree.await?;

    for (name, subdirectory) in subdirectories {
        let subevents = directory_tree_to_events_metadata(dir.clone(), *subdirectory).await?;
        for event in subevents {
            match event.name.as_str() {
                "event" => {
                    events.0.push(EventMetadata {
                        name: name.to_string().to_uppercase().into(),
                        file_path: event.file_path,
                    });
                }
                _ => {
                    let name = match is_group_route(&name) {
                        true => event.name.clone().to_uppercase(),
                        false => format!("{}_{}", name.to_uppercase(), event.name.to_uppercase()),
                    };
                    events.0.push(EventMetadata {
                        name: name.into(),
                        file_path: event.file_path,
                    });
                }
            }
        }
    }

    for (name, file_path) in &components.await?.0 {
        events.0.push(EventMetadata {
            name: name.to_string().to_uppercase().into(),
            file_path: *file_path,
        });
    }

    let mut seen_conflicts: BTreeMap<RcStr, EventMetadata> = Default::default();
    for (name, event) in events.0.iter().enumerate() {
        for (other_name, other_event) in events.0.iter().enumerate() {
            if name != other_name
                && event.name == other_event.name
                && !seen_conflicts.contains_key(&event.name)
            {
                conflict_issue(
                    dir.clone(),
                    event.clone().name.into(),
                    &event.file_path.realpath().await?.to_string(),
                    &other_event.file_path.realpath().await?.to_string(),
                );
                seen_conflicts.insert(event.name.clone(), event.clone());
            }
        }
    }

    Ok(events.cell())
}

fn is_group_route(name: &str) -> bool {
    name.starts_with('(') && name.ends_with(')')
}

fn conflict_issue(dir: Vc<FileSystemPath>, e: String, value_a: &String, value_b: &String) {
    DirectoryTreeIssue {
        dir,
        message: StyledString::Text(
            format!("Conflicting events for {}: {value_a} and {value_b}", e,).into(),
        )
        .cell(),
        severity: IssueSeverity::Fatal.cell(),
    }
    .cell()
    .emit();
}
