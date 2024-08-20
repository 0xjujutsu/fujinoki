use std::path::MAIN_SEPARATOR;

use anyhow::{anyhow, Result};
use fujinoki_core::{
    config::FujinokiConfig,
    structures::{
        commands::{directory_tree_to_commands_metadata, find_commands_dir, CommandsMetadata},
        events::{directory_tree_to_events_metadata, find_events_dir, EventsMetadata},
        get_directory_tree,
    },
};
use fujinoki_websocket::source::{ContentSourceData, EntryMap, Executor};
use turbopack_binding::{
    turbo::{
        tasks as turbo_tasks,
        tasks::{RcStr, TransientInstance, TryJoinIterExt, Value, ValueToString, Vc},
        tasks_env::ProcessEnv,
        tasks_fs::{FileSystem, FileSystemPath},
    },
    turbopack::{
        core::{
            chunk::ChunkingContext,
            context::AssetContext,
            module::Module,
            reference_type::{EntryReferenceSubType, ReferenceType},
            resolve::{
                origin::{PlainResolveOrigin, ResolveOriginExt},
                parse::Request,
                pattern::Pattern,
            },
        },
        ecmascript::EcmascriptModuleAsset,
        ecmascript_runtime::RuntimeType,
        env::dotenv::load_env,
        node::execution_context::ExecutionContext,
        nodejs::NodeJsChunkingContext,
        turbopack::evaluate_context::node_build_environment,
    },
};

use crate::{
    contexts::{get_asset_context, get_compile_time_info, NodeEnv},
    util::{
        commands_metadata_to_entry_requests, events_metadata_to_entry_requests, output_fs,
        project_fs, EntryRequest,
    },
};

#[turbo_tasks::value]
#[derive(Clone, Debug)]
pub struct SourcePreparations {
    pub project_path: Vc<FileSystemPath>,
    pub asset_context: Vc<Box<dyn AssetContext>>,
    pub chunking_context: Vc<Box<dyn ChunkingContext>>,
    pub process_env: Vc<Box<dyn ProcessEnv>>,
}

#[turbo_tasks::function]
pub async fn get_project_path(root_dir: RcStr, project_dir: RcStr) -> Vc<FileSystemPath> {
    let project_relative = project_dir.strip_prefix(&root_dir.to_string()).unwrap();
    let project_relative = project_relative
        .strip_prefix(MAIN_SEPARATOR)
        .unwrap_or(project_relative)
        .replace(MAIN_SEPARATOR, "/");

    let fs = project_fs(root_dir, vec![], true);

    fs.root().join(project_relative.into())
}

#[turbo_tasks::function]
pub async fn source(
    root_dir: RcStr,
    project_dir: RcStr,
    entry_requests: TransientInstance<Vec<EntryRequest>>,
    config: Vc<FujinokiConfig>,
) -> Result<Vc<ContentSourceData>> {
    let output_fs = output_fs(project_dir.clone());
    let build_output_root = output_fs.root().join(".turbopack/build".into());

    let project_path = get_project_path(root_dir, project_dir);

    let process_env: Vc<Box<dyn ProcessEnv>> = load_env(project_path);
    let build_env = node_build_environment();
    let node_env = NodeEnv::Development.cell();

    let build_chunking_context = NodeJsChunkingContext::builder(
        project_path,
        build_output_root,
        build_output_root,
        build_output_root.join("chunks".into()),
        build_output_root.join("assets".into()),
        build_env,
        RuntimeType::Development,
    )
    .build();

    let compile_time_info = get_compile_time_info(node_env);
    let execution_context = ExecutionContext::new(
        project_path,
        Vc::upcast(build_chunking_context),
        process_env,
    );
    let asset_context = get_asset_context(project_path, execution_context, compile_time_info);

    let mut entry_requests = entry_requests.iter().map(|r| r.clone()).collect::<Vec<_>>();
    let mut events_metadata = EventsMetadata::default().cell();
    let mut commands_metadata = CommandsMetadata::default().cell();

    let events_dir = find_events_dir(project_path);
    if let Some(events_dir) = *events_dir.await? {
        let directory_tree = get_directory_tree(events_dir.clone(), config.file_extensions());
        events_metadata = directory_tree_to_events_metadata(events_dir, directory_tree);
        let new_entry_requests = events_metadata_to_entry_requests(events_metadata).await?;
        for entry in new_entry_requests.iter() {
            let entry = entry.clone().await?;
            entry_requests.push(entry.clone_value());
        }
    }

    let commands_dir = find_commands_dir(project_path);
    if let Some(commands_dir) = *commands_dir.await? {
        let directory_tree = get_directory_tree(commands_dir.clone(), config.file_extensions());
        commands_metadata = directory_tree_to_commands_metadata(commands_dir, directory_tree);
        let new_entry_requests = commands_metadata_to_entry_requests(commands_metadata).await?;
        for entry in new_entry_requests.iter() {
            let entry = entry.clone().await?;
            entry_requests.push(entry.clone_value());
        }
    }

    let entry_requests: Vec<Vc<Request>> = entry_requests
        .iter()
        .map(|r| match r {
            EntryRequest::Relative(p) => Request::relative(
                Value::new(Pattern::Constant(p.clone().into())),
                Default::default(),
                Default::default(),
                false,
            ),
            EntryRequest::Module(m, p) => Request::module(
                m.clone().into(),
                Value::new(Pattern::Constant(p.clone().into())),
                Default::default(),
                Default::default(),
            ),
        })
        .collect();

    let origin = PlainResolveOrigin::new(asset_context, project_path);
    let entries = entry_requests
        .into_iter()
        .map(|request| async move {
            let ty = Value::new(ReferenceType::Entry(EntryReferenceSubType::AppPage));
            Ok(origin
                .resolve_asset(request, origin.resolve_options(ty.clone()), ty)
                .primary_modules()
                .await?
                .first()
                .copied())
        })
        .try_join()
        .await?;

    let entries: Vec<_> = entries
        .into_iter()
        .flatten()
        .map(|module| async move {
            if let Some(ecmascript) =
                Vc::try_resolve_downcast_type::<EcmascriptModuleAsset>(module).await?
            {
                Ok(Vc::upcast(ecmascript))
            } else if let Some(chunkable) =
                Vc::try_resolve_sidecast::<Box<dyn Module>>(module).await?
            {
                // TODO(turbopack) this is missing runtime code, so it's probably broken and we
                // should also add an ecmascript chunk with the runtime code
                Ok(chunkable)
            } else {
                // TODO(turbopack) convert into a serve-able asset
                Err(anyhow!(
                    "Entry module is not chunkable, so it can't be used to bootstrap the \
                     application"
                ))
            }
        })
        .try_join()
        .await?;

    let mut mapped_entries = EntryMap::empty();

    for entry in entries.clone() {
        mapped_entries.insert_entry(entry.ident().path().to_string().await?.clone_value(), entry);
    }

    Ok((ContentSourceData {
        project_path: project_path.clone(),
        // TODO replace this with something more practical
        executor: Executor::new(
            project_path,
            process_env,
            asset_context,
            Vc::upcast(build_chunking_context),
            None,
        ),
        entries: mapped_entries.cell(),
        events: events_metadata,
        commands: commands_metadata,
        events_dir,
        commands_dir,
    })
    .cell())
}
