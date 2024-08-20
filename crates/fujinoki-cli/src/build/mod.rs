use std::{
    collections::HashSet,
    env::current_dir,
    path::{PathBuf, MAIN_SEPARATOR},
    sync::Arc,
};

use anyhow::{bail, Context, Result};
use fujinoki_cli_utils::issue::{ConsoleUi, LogOptions};
use fujinoki_core::{
    config::FujinokiConfig,
    structures::{
        commands::{directory_tree_to_commands_metadata, find_commands_dir},
        events::{directory_tree_to_events_metadata, find_events_dir},
        get_directory_tree,
    },
};
use turbopack_binding::{
    turbo::{
        tasks as turbo_tasks,
        tasks::{RcStr, TransientInstance, TryJoinIterExt, TurboTasks, Value, Vc},
        tasks_fs::FileSystem,
        tasks_memory::MemoryBackend,
    },
    turbopack::{
        core::{
            asset::Asset,
            chunk::{
                availability_info::AvailabilityInfo, ChunkableModule, ChunkingContext,
                ChunkingContextExt, EvaluatableAssets, MinifyType,
            },
            issue::{handle_issues, IssueReporter, IssueSeverity},
            module::Module,
            output::OutputAsset,
            reference::all_assets_from_entries,
            reference_type::{EntryReferenceSubType, ReferenceType},
            resolve::{
                origin::{PlainResolveOrigin, ResolveOriginExt},
                parse::Request,
                pattern::Pattern,
            },
        },
        ecmascript_runtime::RuntimeType,
        env::dotenv::load_env,
        node::execution_context::ExecutionContext,
        nodejs::NodeJsChunkingContext,
        turbopack::{ecmascript::EcmascriptModuleAsset, evaluate_context::node_build_environment},
    },
};

use crate::{
    arguments::BuildArguments,
    contexts::{get_asset_context, get_compile_time_info, NodeEnv},
    util::{
        commands_metadata_to_entry_requests, events_metadata_to_entry_requests, normalize_dirs,
        output_fs, project_fs, EntryRequest, EntryRequests, NormalizedDirs,
    },
};

pub fn register() {
    turbopack_binding::turbopack::turbopack::register();
    turbopack_binding::turbopack::nodejs::register();
    fujinoki_cli_utils::register();
    fujinoki_core::register();
}

pub struct FujinokiBuildBuilder {
    turbo_tasks: Arc<TurboTasks<MemoryBackend>>,
    project_dir: String,
    root_dir: String,
    entry_requests: Vec<EntryRequest>,
    log_level: IssueSeverity,
    show_all: bool,
    log_detail: bool,
    minify_type: MinifyType,
}

impl FujinokiBuildBuilder {
    pub fn new(
        turbo_tasks: Arc<TurboTasks<MemoryBackend>>,
        project_dir: String,
        root_dir: String,
    ) -> Self {
        FujinokiBuildBuilder {
            turbo_tasks,
            project_dir,
            root_dir,
            entry_requests: vec![],
            log_level: IssueSeverity::Warning,
            show_all: false,
            log_detail: false,
            minify_type: MinifyType::Minify,
        }
    }

    pub fn log_level(mut self, log_level: IssueSeverity) -> Self {
        self.log_level = log_level;
        self
    }

    pub fn show_all(mut self, show_all: bool) -> Self {
        self.show_all = show_all;
        self
    }

    pub fn log_detail(mut self, log_detail: bool) -> Self {
        self.log_detail = log_detail;
        self
    }

    pub fn minify_type(mut self, minify_type: MinifyType) -> Self {
        self.minify_type = minify_type;
        self
    }

    pub async fn build(self) -> Result<()> {
        let task = self.turbo_tasks.spawn_once_task::<(), _>(async move {
            let build_result = build_internal(
                self.project_dir.clone().into(),
                self.root_dir.into(),
                EntryRequests(
                    self.entry_requests
                        .iter()
                        .cloned()
                        .map(EntryRequest::cell)
                        .collect(),
                )
                .cell(),
                self.minify_type,
            );

            // Await the result to propagate any errors.
            build_result.await?;

            let issue_reporter: Vc<Box<dyn IssueReporter>> =
                Vc::upcast(ConsoleUi::new(TransientInstance::new(LogOptions {
                    project_dir: PathBuf::from(self.project_dir),
                    current_dir: current_dir().unwrap(),
                    show_all: self.show_all,
                    log_detail: self.log_detail,
                    log_level: self.log_level,
                })));

            handle_issues(
                build_result,
                issue_reporter,
                IssueSeverity::Error.into(),
                None,
                None,
            )
            .await?;

            Ok(Default::default())
        });

        self.turbo_tasks.wait_task_completion(task, true).await?;

        Ok(())
    }
}

#[turbo_tasks::function]
async fn build_internal(
    project_dir: RcStr,
    root_dir: RcStr,
    entry_requests: Vc<EntryRequests>,
    minify_type: MinifyType,
) -> Result<Vc<()>> {
    let env = node_build_environment();
    let output_fs = output_fs(project_dir.clone());
    let project_fs = project_fs(root_dir.clone(), Default::default(), false);
    let project_relative = project_dir.strip_prefix(&root_dir.to_string()).unwrap();
    let project_relative = project_relative
        .strip_prefix(MAIN_SEPARATOR)
        .unwrap_or(project_relative)
        .replace(MAIN_SEPARATOR, "/");
    let project_path = project_fs.root().join(project_relative.into());
    let build_output_root = output_fs.root().join(".turbopack/build".to_string().into());

    let node_env: Vc<NodeEnv> = NodeEnv::Production.cell();

    // TODO use the actual config
    dbg!("USING FAKE CONFIG");

    let config = FujinokiConfig::from_string(
        Vc::cell("{\"file_extensions\":[\"js\",\"ts\",\"jsx\",\"tsx\"]}".into()),
        Some(NodeEnv::Production.to_string().into()),
    );
    // let config = FujinokiConfig::validate(config);

    let mut entry_requests = entry_requests
        .await?
        .iter()
        .map(|r| r.clone())
        .collect::<Vec<_>>();

    let events_dir = find_events_dir(project_path);
    if let Some(events_dir) = &*events_dir.await? {
        let directory_tree = get_directory_tree(*events_dir, config.file_extensions());
        let events_metadata = directory_tree_to_events_metadata(*events_dir, directory_tree);
        let new_entry_requests = events_metadata_to_entry_requests(events_metadata).await?;
        for entry in new_entry_requests.iter() {
            entry_requests.push(*entry);
        }
    }

    let commands_dir = find_commands_dir(project_path);
    if let Some(commands_dir) = &*commands_dir.await? {
        let directory_tree = get_directory_tree(*commands_dir, config.file_extensions());
        let commands_metadata = directory_tree_to_commands_metadata(*commands_dir, directory_tree);
        let new_entry_requests = commands_metadata_to_entry_requests(commands_metadata).await?;
        for entry in new_entry_requests.iter() {
            entry_requests.push(*entry);
        }
    }

    let chunking_context = Vc::upcast(
        NodeJsChunkingContext::builder(
            project_path,
            build_output_root,
            build_output_root,
            build_output_root.join("chunks".to_string().into()),
            build_output_root.join("assets".to_string().into()),
            env,
            RuntimeType::Production,
        )
        .minify_type(minify_type)
        .build(),
    );

    let compile_time_info = get_compile_time_info(node_env);
    let execution_context =
        ExecutionContext::new(project_path, chunking_context, load_env(project_path));
    let asset_context = get_asset_context(project_path, execution_context, compile_time_info);

    let entry_requests = (*entry_requests
        .iter()
        .cloned()
        .map(|r| async move {
            Ok(match &*r.await? {
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
        })
        .try_join()
        .await?)
        .to_vec();

    let origin =
        PlainResolveOrigin::new(asset_context, output_fs.root().join("_".to_string().into()));
    let project_dir = &project_dir;
    let entries = entry_requests
        .into_iter()
        .map(|request_vc| async move {
            let ty = Value::new(ReferenceType::Entry(EntryReferenceSubType::Undefined));
            let request = request_vc.await?;
            origin
                .resolve_asset(request_vc, origin.resolve_options(ty.clone()), ty)
                .first_module()
                .await?
                .with_context(|| {
                    format!(
                        "Unable to resolve entry {} from directory {}.",
                        request.request().unwrap(),
                        project_dir
                    )
                })
        })
        .try_join()
        .await?;

    let entry_chunk_groups = entries
        .into_iter()
        .map(|entry_module| async move {
            Ok(
                if let Some(ecmascript) =
                    Vc::try_resolve_downcast_type::<EcmascriptModuleAsset>(entry_module).await?
                {
                    Vc::cell(vec![
                        Vc::try_resolve_downcast_type::<NodeJsChunkingContext>(chunking_context)
                            .await?
                            .unwrap()
                            .entry_chunk_group(
                                build_output_root
                                    .join(
                                        ecmascript
                                            .ident()
                                            .path()
                                            .parent()
                                            .file_stem()
                                            .await?
                                            .as_deref()
                                            .unwrap()
                                            .to_string()
                                            .into(),
                                    )
                                    .join(
                                        ecmascript
                                            .ident()
                                            .path()
                                            .file_stem()
                                            .await?
                                            .as_deref()
                                            .unwrap()
                                            .to_string()
                                            .into(),
                                    )
                                    .with_extension("js".to_string().into()),
                                Vc::upcast(ecmascript),
                                EvaluatableAssets::one(Vc::upcast(ecmascript)),
                                Value::new(AvailabilityInfo::Root),
                            )
                            .await?
                            .asset,
                    ])
                } else if let Some(chunkable) =
                    Vc::try_resolve_sidecast::<Box<dyn ChunkableModule>>(entry_module).await?
                {
                    chunking_context.root_chunk_group_assets(chunkable)
                } else {
                    // TODO convert into a serve-able asset
                    bail!(
                        "Entry module is not chunkable, so it can't be used to bootstrap the \
                         application"
                    )
                },
            )
        })
        .try_join()
        .await?;

    let mut chunks: HashSet<Vc<Box<dyn OutputAsset>>> = HashSet::new();
    for chunk_group in entry_chunk_groups {
        chunks.extend(&*all_assets_from_entries(chunk_group).await?);
    }

    chunks
        .iter()
        .map(|c| c.content().write(c.ident().path()))
        .try_join()
        .await?;

    Ok(Default::default())
}

pub async fn build(args: &BuildArguments) -> Result<()> {
    register();

    let NormalizedDirs {
        project_dir,
        root_dir,
    } = normalize_dirs(&args.common.dir, &args.common.root)?;

    let tt = TurboTasks::new(MemoryBackend::new(
        args.turbo
            .memory_limit
            .map_or(usize::MAX, |l| l * 1024 * 1024),
    ));

    let builder = FujinokiBuildBuilder::new(tt.clone(), project_dir.clone(), root_dir)
        .log_detail(args.turbo.log_detail)
        .log_level(
            args.turbo
                .log_level
                .map_or_else(|| IssueSeverity::Warning, |l| l.0),
        )
        .minify_type(if args.no_minify {
            MinifyType::NoMinify
        } else {
            MinifyType::Minify
        })
        .show_all(args.turbo.show_all);

    builder.build().await?;

    Ok(())
}
