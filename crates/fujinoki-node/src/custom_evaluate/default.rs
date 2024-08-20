use std::{borrow::Cow, thread::available_parallelism};

use anyhow::{bail, Result};
use async_trait::async_trait;
use indexmap::indexmap;
use serde::{Deserialize, Serialize};
use turbopack_binding::turbo::tasks::{Completion, TaskInput, Value, Vc};
use turbopack_binding::{
    turbo::{
        tasks_env::ProcessEnv,
        tasks_fs::{to_sys_path, File, FileSystemPath},
    },
    turbopack::{
        core::{
            asset::AssetContent,
            chunk::{ChunkingContext, ChunkingContextExt, EvaluatableAsset, EvaluatableAssets},
            context::AssetContext,
            file_source::FileSource,
            ident::AssetIdent,
            issue::IssueExt,
            module::Module,
            reference_type::{InnerAssets, ReferenceType},
            virtual_source::VirtualSource,
        },
        node::debug::should_debug,
        resolve::resolve_options_context::ResolveOptionsContext,
    },
};
use turbopack_binding::turbo::tasks as turbo_tasks;

use crate::{
    embed_js::embed_file_path,
    emit, emit_package_json,
    evaluate::{compute, EvaluateContext, EvaluationIssue, JavaScriptStreamSender},
    internal_assets_for_source_mapping,
    pool::NodeJsPool,
    source_map::StructuredError,
};

#[turbo_tasks::function]
async fn default_value_compute(
    evaluate_context: DefaultValueEvaluateContext,
    sender: Vc<JavaScriptStreamSender>,
) -> Result<Vc<()>> {
    compute(evaluate_context, sender).await
}

#[derive(Clone, PartialEq, Eq, Hash, TaskInput, Debug, Serialize, Deserialize)]
pub struct DefaultValueEvaluateContext {
    pub module_asset: Vc<Box<dyn Module>>,
    pub cwd: Vc<FileSystemPath>,
    pub env: Vc<Box<dyn ProcessEnv>>,
    pub context_ident_for_issue: Vc<AssetIdent>,
    pub asset_context: Vc<Box<dyn AssetContext>>,
    pub chunking_context: Vc<Box<dyn ChunkingContext>>,
    pub resolve_options_context: Option<Vc<ResolveOptionsContext>>,
    pub additional_invalidation: Vc<Completion>,
}

#[async_trait]
impl EvaluateContext for DefaultValueEvaluateContext {
    type InfoMessage = ();
    type RequestMessage = ();
    type ResponseMessage = ();
    type State = ();

    fn compute(self, sender: Vc<JavaScriptStreamSender>) {
        let _ = default_value_compute(self, sender);
    }

    fn pool(&self) -> Vc<NodeJsPool> {
        get_evaluate_pool(
            self.module_asset,
            self.cwd,
            self.env,
            self.asset_context,
            self.chunking_context,
            None,
            self.additional_invalidation,
            should_debug("evaluate_default_value"),
        )
    }

    fn args(&self) -> &[Vc<serde_json::Value>] {
        &[]
    }

    fn cwd(&self) -> Vc<FileSystemPath> {
        self.cwd
    }

    fn keep_alive(&self) -> bool {
        true
    }

    async fn emit_error(&self, error: StructuredError, pool: &NodeJsPool) -> Result<()> {
        EvaluationIssue {
            error: error,
            context_ident: self.context_ident_for_issue,
            assets_for_source_mapping: pool.assets_for_source_mapping,
            assets_root: pool.assets_root,
            project_dir: self.chunking_context.context_path().root(),
        }
        .cell()
        .emit();
        Ok(())
    }

    async fn info(
        &self,
        _state: &mut Self::State,
        _data: Self::InfoMessage,
        _pool: &NodeJsPool,
    ) -> Result<()> {
        bail!("DefaultValueEvaluateContext does not support info messages")
    }

    async fn request(
        &self,
        _state: &mut Self::State,
        _data: Self::RequestMessage,
        _pool: &NodeJsPool,
    ) -> Result<Self::ResponseMessage> {
        bail!("DefaultValueEvaluateContext does not support request messages")
    }

    async fn finish(&self, _state: Self::State, _pool: &NodeJsPool) -> Result<()> {
        Ok(())
    }
}

#[turbo_tasks::function]
/// Pass the file you cared as `runtime_entries` to invalidate and reload the
/// evaluated result automatically.
pub async fn get_evaluate_pool(
    module_asset: Vc<Box<dyn Module>>,
    cwd: Vc<FileSystemPath>,
    env: Vc<Box<dyn ProcessEnv>>,
    asset_context: Vc<Box<dyn AssetContext>>,
    chunking_context: Vc<Box<dyn ChunkingContext>>,
    runtime_entries: Option<Vc<EvaluatableAssets>>,
    additional_invalidation: Vc<Completion>,
    debug: bool,
) -> Result<Vc<NodeJsPool>> {
    let runtime_asset = asset_context
        .process(
            Vc::upcast(FileSource::new(embed_file_path("ipc/evaluate.ts".into()))),
            Value::new(ReferenceType::Internal(InnerAssets::empty())),
        )
        .module();

    let module_path = module_asset.ident().path().await?;
    let file_name = module_path.file_name();
    let file_name = if file_name.ends_with(".js") {
        Cow::Borrowed(file_name)
    } else if let Some(file_name) = file_name.strip_suffix(".ts") {
        Cow::Owned(format!("{file_name}.js"))
    } else {
        Cow::Owned(format!("{file_name}.js"))
    };
    let path = chunking_context.output_root().join(file_name.into());
    let entry_module = asset_context
        .process(
            Vc::upcast(VirtualSource::new(
                runtime_asset.ident().path().join("evaluate.js".into()),
                AssetContent::file(
                    File::from(
                        "import { getDefaultValue } from 'RUNTIME'; getDefaultValue(() => \
                         import('INNER'))"
                            .to_string(),
                    )
                    .into(),
                ),
            )),
            Value::new(ReferenceType::Internal(Vc::cell(indexmap! {
                "INNER".into() => module_asset,
                "RUNTIME".into() => runtime_asset
            }))),
        )
        .module();

    let (Some(cwd), Some(entrypoint)) = (to_sys_path(cwd).await?, to_sys_path(path).await?) else {
        panic!("can only evaluate from a disk filesystem");
    };

    let runtime_entries = {
        let globals_module = asset_context
            .process(
                Vc::upcast(FileSource::new(embed_file_path("globals.ts".into()))),
                Value::new(ReferenceType::Internal(InnerAssets::empty())),
            )
            .module();

        let Some(globals_module) =
            Vc::try_resolve_sidecast::<Box<dyn EvaluatableAsset>>(globals_module).await?
        else {
            bail!("Internal module is not evaluatable");
        };

        let mut entries = vec![globals_module];
        if let Some(runtime_entries) = runtime_entries {
            for &entry in &*runtime_entries.await? {
                entries.push(entry)
            }
        }

        Vc::<EvaluatableAssets>::cell(entries)
    };

    let bootstrap =
        chunking_context.root_entry_chunk_group_asset(path, entry_module, runtime_entries);

    let output_root: Vc<FileSystemPath> = chunking_context.output_root();
    let emit_package = emit_package_json(output_root);
    let emit = emit(bootstrap, output_root);
    let assets_for_source_mapping = internal_assets_for_source_mapping(bootstrap, output_root);
    emit_package.await?;
    emit.await?;
    let pool = NodeJsPool::new(
        cwd,
        entrypoint,
        env.read_all()
            .await?
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        assets_for_source_mapping,
        output_root,
        chunking_context.context_path().root(),
        available_parallelism().map_or(1, |v| v.get()),
        debug,
    );
    additional_invalidation.await?;
    Ok(pool.cell())
}
