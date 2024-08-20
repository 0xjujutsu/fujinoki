use std::{collections::HashMap, fmt};

use anyhow::Result;
use fujinoki_ecmascript_plugins::after_resolve::external_cjs_modules::{
    ExternalCjsModulesResolvePlugin, ExternalPredicate,
};
use turbopack_binding::{
    turbo::{
        tasks as turbo_tasks,
        tasks::Vc,
        tasks_fs::{FileSystem, FileSystemPath},
    },
    turbopack::{
        core::{
            compile_time_defines,
            compile_time_info::{CompileTimeDefines, CompileTimeInfo},
            context::AssetContext,
            environment::Environment,
            resolve::options::{ImportMap, ImportMapping},
        },
        ecmascript::TreeShakingMode,
        node::execution_context::ExecutionContext,
        turbopack::{
            condition::ContextCondition,
            evaluate_context::node_build_environment,
            module_options::{JsxTransformOptions, ModuleOptionsContext},
            resolve_options_context::ResolveOptionsContext,
            ModuleAssetContext,
        },
    },
};

use crate::util::{get_existing_tsconfigs, tsconfig_compiler_options};

#[turbo_tasks::value(shared)]
pub enum NodeEnv {
    Development,
    Production,
}

impl fmt::Display for NodeEnv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeEnv::Development => f.write_str("development"),
            NodeEnv::Production => f.write_str("production"),
        }
    }
}

async fn foreign_code_context_condition() -> Result<ContextCondition> {
    Ok(ContextCondition::InDirectory("node_modules".to_string()))
}

#[turbo_tasks::function]
pub async fn get_import_map(project_path: Vc<FileSystemPath>) -> Result<Vc<ImportMap>> {
    let mut import_map = ImportMap::empty();

    import_map.insert_singleton_alias("@swc/helpers", project_path);

    import_map.insert_wildcard_alias(
        "@vercel/turbopack-ecmascript-runtime/",
        ImportMapping::PrimaryAlternative(
            "./*".to_string().into(),
            Some(turbopack_binding::turbopack::ecmascript_runtime::embed_fs().root()),
        )
        .cell(),
    );

    Ok(import_map.cell())
}

#[turbo_tasks::function]
pub async fn get_resolve_options_context(
    project_path: Vc<FileSystemPath>,
) -> Result<Vc<ResolveOptionsContext>> {
    let external_cjs_modules_plugin = ExternalCjsModulesResolvePlugin::new(
        project_path,
        project_path.root(),
        ExternalPredicate::AllExcept(Default::default()).cell(),
        true,
    );

    let import_map = get_import_map(project_path);

    let module_options_context = ResolveOptionsContext {
        enable_node_modules: Some(project_path.root().resolve().await?),
        enable_node_native_modules: true,
        enable_node_externals: true,
        custom_conditions: vec![],
        import_map: Some(import_map),
        browser: false,
        ..Default::default()
    };

    Ok(ResolveOptionsContext {
        enable_typescript: true,
        enable_react: true,
        rules: vec![(
            foreign_code_context_condition().await?,
            module_options_context.clone().cell(),
        )],
        after_resolve_plugins: vec![Vc::upcast(external_cjs_modules_plugin)],
        ..module_options_context
    }
    .cell())
}

#[turbo_tasks::function]
async fn get_module_options_context(
    execution_context: Vc<ExecutionContext>,
    env: Vc<Environment>,
) -> Result<Vc<ModuleOptionsContext>> {
    let module_options_context = ModuleOptionsContext {
        preset_env_versions: Some(env),
        execution_context: Some(execution_context),
        tree_shaking_mode: Some(TreeShakingMode::ReexportsOnly),
        import_externals: true,
        ..Default::default()
    };

    let tsconfig_path: Option<Vc<FileSystemPath>> =
        *get_existing_tsconfigs(execution_context.project_path()).await?;
    let tsconfig_compiler_options = if let Some(tsconfig_path) = tsconfig_path {
        if let Some(compiler_options) = &*tsconfig_compiler_options(tsconfig_path).await? {
            Some(compiler_options.await?)
        } else {
            None
        }
    } else {
        None
    };

    let enable_jsx = Some(
        JsxTransformOptions {
            // TODO development: bool
            import_source: tsconfig_compiler_options
                .map(|o| {
                    o.to_owned()
                        .get("jsxImportSource")
                        .unwrap()
                        .as_str()
                        .map(|s| s.to_string().into())
                })
                .unwrap_or_default(),
            ..Default::default()
        }
        .cell(),
    );

    let module_options_context = ModuleOptionsContext {
        enable_jsx,
        enable_postcss_transform: None,
        enable_typescript_transform: Some(Default::default()),
        import_externals: true,
        rules: vec![(
            foreign_code_context_condition().await?,
            module_options_context.clone().cell(),
        )],
        ..module_options_context
    }
    .cell();

    Ok(module_options_context)
}

#[turbo_tasks::function]
pub fn get_asset_context(
    project_path: Vc<FileSystemPath>,
    execution_context: Vc<ExecutionContext>,
    compile_time_info: Vc<CompileTimeInfo>,
) -> Vc<Box<dyn AssetContext>> {
    let resolve_options_context = get_resolve_options_context(project_path);
    let module_options_context =
        get_module_options_context(execution_context, compile_time_info.environment());

    let asset_context: Vc<Box<dyn AssetContext>> = Vc::upcast(ModuleAssetContext::new(
        Vc::cell(HashMap::new()),
        compile_time_info,
        module_options_context,
        resolve_options_context,
        Vc::cell("server".to_string().into()),
    ));

    asset_context
}

fn client_defines(node_env: &NodeEnv) -> Vc<CompileTimeDefines> {
    compile_time_defines!(
        process.turbopack = true,
        process.fujinoki = true,
        process.env.FUJINOKI = true,
        process.env.TURBOPACK = true,
        process.env.NODE_ENV = node_env.to_string()
    )
    .cell()
}

#[turbo_tasks::function]
pub async fn get_compile_time_info(node_env: Vc<NodeEnv>) -> Result<Vc<CompileTimeInfo>> {
    Ok(CompileTimeInfo::builder(node_build_environment())
        .defines(client_defines(&*node_env.await?))
        .cell())
}
