use std::{env::current_dir, path::PathBuf};

use anyhow::{Context, Result};
use dunce::canonicalize;
use fujinoki_core::structures::{commands::CommandsMetadata, events::EventsMetadata};
use serde_json::Value as JsonValue;
use turbopack_binding::{
    turbo::{
        tasks as turbo_tasks,
        tasks::{RcStr, Value, Vc},
        tasks_fs::{DiskFileSystem, FileContent, FileSystem, FileSystemPath},
    },
    turbopack::{
        core::{
            context::{AssetContext, ProcessResult},
            file_source::FileSource,
            reference_type::{EntryReferenceSubType, ReferenceType},
            resolve::node::node_cjs_resolve_options,
            PROJECT_FILESYSTEM_NAME,
        },
        ecmascript::typescript::resolve::{read_from_tsconfigs, read_tsconfigs, tsconfig},
    },
};

#[turbo_tasks::value(transparent)]
pub struct EntryRequests(pub Vec<Vc<EntryRequest>>);

#[turbo_tasks::value(shared)]
#[derive(Clone, Debug)]
pub enum EntryRequest {
    Relative(String),
    Module(String, String),
}

pub struct NormalizedDirs {
    /// Normalized project directory path as an absolute path
    pub project_dir: String,
    /// Normalized root directory path as an absolute path
    pub root_dir: String,
}

/// Normalizes (canonicalizes and represents as an absolute path in a String)
/// the project and root directories.
pub fn normalize_dirs(
    project_dir: &Option<PathBuf>,
    root_dir: &Option<PathBuf>,
) -> Result<NormalizedDirs> {
    let project_dir = project_dir
        .as_ref()
        .map(canonicalize)
        .unwrap_or_else(current_dir)
        .context("project directory can't be found")?
        .to_str()
        .context("project directory contains invalid characters")?
        .to_string();

    let root_dir = match root_dir.as_ref() {
        Some(root) => canonicalize(root)
            .context("root directory can't be found")?
            .to_str()
            .context("root directory contains invalid characters")?
            .to_string(),
        None => project_dir.clone(),
    };

    Ok(NormalizedDirs {
        project_dir,
        root_dir,
    })
}

#[turbo_tasks::function]
pub async fn project_fs(
    project_dir: RcStr,
    ignored_subpaths: Vec<RcStr>,
    watching: bool,
) -> Result<Vc<Box<dyn FileSystem>>> {
    let disk_fs = DiskFileSystem::new(
        PROJECT_FILESYSTEM_NAME.into(),
        project_dir,
        ignored_subpaths,
    );
    if watching {
        disk_fs.await?.start_watching_with_invalidation_reason()?;
    }
    Ok(Vc::upcast(disk_fs))
}

#[turbo_tasks::function]
pub async fn output_fs(project_dir: RcStr) -> Result<Vc<Box<dyn FileSystem>>> {
    let disk_fs = DiskFileSystem::new("output".to_string().into(), project_dir, vec![]);
    disk_fs.await?.start_watching()?;
    Ok(Vc::upcast(disk_fs))
}

#[turbo_tasks::function]
pub async fn events_metadata_to_entry_requests(
    events_metadata: Vc<EventsMetadata>,
) -> Result<Vc<EntryRequests>> {
    let mut entry_requests: EntryRequests = EntryRequests { 0: vec![] };

    for metadata in events_metadata.await? {
        entry_requests
            .0
            .push(EntryRequest::Relative(metadata.file_path.realpath().await?.to_string()).cell());
    }

    Ok(entry_requests.cell())
}

#[turbo_tasks::function]
pub async fn commands_metadata_to_entry_requests(
    commands_metadata: Vc<CommandsMetadata>,
) -> Result<Vc<EntryRequests>> {
    let mut entry_requests: EntryRequests = EntryRequests { 0: vec![] };

    for metadata in commands_metadata.await? {
        entry_requests
            .0
            .push(EntryRequest::Relative(metadata.file_path.realpath().await?.to_string()).cell());
    }

    Ok(entry_requests.cell())
}

#[turbo_tasks::function]
fn process_path_to_asset(
    path: Vc<FileSystemPath>,
    asset_context: Vc<Box<dyn AssetContext>>,
) -> Vc<ProcessResult> {
    asset_context.process(
        Vc::upcast(FileSource::new(path)),
        Value::new(ReferenceType::Entry(EntryReferenceSubType::Undefined)),
    )
}

#[turbo_tasks::value(transparent)]
pub struct OptionJsonValue(Option<Vc<JsonValue>>);

/// Returns the compiler options
#[turbo_tasks::function]
pub async fn tsconfig_compiler_options(
    tsconfig_path: Vc<FileSystemPath>,
) -> Result<Vc<OptionJsonValue>> {
    let configs = read_tsconfigs(
        tsconfig_path.read(),
        Vc::upcast(FileSource::new(tsconfig_path)),
        node_cjs_resolve_options(tsconfig_path.root()),
    )
    .await?;

    if configs.is_empty() {
        return Ok(Vc::cell(None));
    }

    let compiler_options = if let Some(compiler_options) =
        read_from_tsconfigs(&configs, |json, _| Some(json["compilerOptions"].clone())).await?
    {
        Some(Vc::cell(compiler_options))
    } else {
        None
    };

    Ok(OptionJsonValue(compiler_options).cell())
}

#[turbo_tasks::function]
pub async fn get_existing_tsconfigs(
    project_path: Vc<FileSystemPath>,
) -> Result<Vc<Option<Vc<FileSystemPath>>>> {
    let tsconfigs = tsconfig().await?;

    for tsconfig in tsconfigs.iter() {
        let tsconfig_path = project_path.append(tsconfig.clone());
        if let FileContent::Content(_) = *tsconfig_path.read().await? {
            return Ok(Vc::cell(Some(tsconfig_path)));
        }
    }

    Ok(Vc::cell(None))
}
