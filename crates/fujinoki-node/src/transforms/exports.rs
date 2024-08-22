use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize, Value as JsonValue};
use turbopack_binding::{
    turbo::{
        tasks::{self as turbo_tasks, Completion, RcStr, TaskInput, Vc},
        tasks_env::ProcessEnv,
        tasks_fs::FileSystemPath,
    },
    turbopack::{
        core::{chunk::ChunkingContext, context::AssetContext, ident::AssetIdent, issue::IssueExt, module::Module},
        node::{
            evaluate::{
                compute, custom_evaluate, EvaluateContext, EvaluationIssue, JavaScriptEvaluation, JavaScriptStreamSender
            },
            NodeJsPool,
        },
        resolve::resolve_options_context::ResolveOptionsContext,
    },
};

#[turbo_tasks::function]
pub(crate) fn evaluate_exports(exports_context: ExportsContext) -> Vc<JavaScriptEvaluation> {
    custom_evaluate(exports_context)
}

#[turbo_tasks::function]
async fn compute_exports_evaluation(
    exports_context: ExportsContext,
    sender: Vc<JavaScriptStreamSender>,
) -> Result<Vc<()>> {
    compute(exports_context, sender).await
}

#[derive(Clone, PartialEq, Eq, Hash, TaskInput, Serialize, Deserialize, Debug)]
pub struct ExportsContext {
    pub module_asset: Vc<Box<dyn Module>>,
    pub cwd: Vc<FileSystemPath>,
    pub env: Vc<Box<dyn ProcessEnv>>,
    pub context_ident_for_issue: Vc<AssetIdent>,
    pub asset_context: Vc<Box<dyn AssetContext>>,
    pub chunking_context: Vc<Box<dyn ChunkingContext>>,
    pub resolve_options_context: Option<Vc<ResolveOptionsContext>>,

    pub args: Vec<Vc<RcStr>>,
    pub additional_invalidation: Vc<Completion>,
}

#[async_trait]
impl EvaluateContext for ExportsContext {
    // TODO
    type InfoMessage = ();
    type RequestMessage = ();
    type ResponseMessage = ();
    type State = ();

    fn compute(self, sender: Vc<JavaScriptStreamSender>) {
        let _ = compute_exports_evaluation(self, sender);
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
            should_debug("exports_loader"),
            self.export_names.clone(),
        )
    }

    fn args(&self) -> &[Vc<JsonValue>] {
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
        todo!()
    }

    async fn request(
        &self,
        _state: &mut Self::State,
        _data: Self::RequestMessage,
        _pool: &NodeJsPool,
    ) -> Result<Self::ResponseMessage> {
        todo!()
    }

    async fn finish(&self, _state: Self::State, _pool: &NodeJsPool) -> Result<()> {
        todo!()
    }
}
