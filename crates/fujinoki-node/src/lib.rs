// TODO(kijv) update this to newest impl of turbopack-node (or try to make
// NodeJsPool public)
#![feature(async_closure)]
#![feature(min_specialization)]
#![feature(lint_reasons)]
#![feature(arbitrary_self_types)]
#![feature(extract_if)]

use std::{collections::HashMap, iter::once};

use anyhow::Result;
use indexmap::IndexSet;
use turbopack_binding::{
    turbo::{
        tasks as turbo_tasks,
        tasks::{
            graph::{AdjacencyMap, GraphTraversal},
            Completion, Completions, TryJoinIterExt, Vc,
        },
        tasks_fs::{File, FileSystemPath},
    },
    turbopack::{
        core::{
            asset::{Asset, AssetContent},
            output::{OutputAsset, OutputAssetsSet},
            source_map::GenerateSourceMap,
            virtual_output::VirtualOutputAsset,
        },
        node::AssetsForSourceMapping,
    },
};

pub mod custom_evaluate;
pub mod embed_js;
pub mod evaluate;
mod pool;
pub mod source_map;

#[turbo_tasks::function]
async fn emit(
    intermediate_asset: Vc<Box<dyn OutputAsset>>,
    intermediate_output_path: Vc<FileSystemPath>,
) -> Result<Vc<Completion>> {
    Ok(Vc::<Completions>::cell(
        internal_assets(intermediate_asset, intermediate_output_path)
            .strongly_consistent()
            .await?
            .iter()
            .map(|a| a.content().write(a.ident().path()))
            .collect(),
    )
    .completed())
}

/// List of the all assets of the "internal" subgraph and a list of boundary
/// assets that are not considered "internal" ("external")
#[derive(Debug)]
#[turbo_tasks::value]
struct SeparatedAssets {
    internal_assets: Vc<OutputAssetsSet>,
    external_asset_entrypoints: Vc<OutputAssetsSet>,
}

/// Extracts the subgraph of "internal" assets (assets within the passes
/// directory). Also lists all boundary assets that are not part of the
/// "internal" subgraph.
#[turbo_tasks::function]
async fn internal_assets(
    intermediate_asset: Vc<Box<dyn OutputAsset>>,
    intermediate_output_path: Vc<FileSystemPath>,
) -> Result<Vc<OutputAssetsSet>> {
    Ok(
        separate_assets(intermediate_asset, intermediate_output_path)
            .strongly_consistent()
            .await?
            .internal_assets,
    )
}

/// Extracts a map of "internal" assets ([`internal_assets`]) which implement
/// the [GenerateSourceMap] trait.
#[turbo_tasks::function]
async fn internal_assets_for_source_mapping(
    intermediate_asset: Vc<Box<dyn OutputAsset>>,
    intermediate_output_path: Vc<FileSystemPath>,
) -> Result<Vc<AssetsForSourceMapping>> {
    let internal_assets = internal_assets(intermediate_asset, intermediate_output_path).await?;
    let intermediate_output_path = &*intermediate_output_path.await?;
    let mut internal_assets_for_source_mapping = HashMap::new();
    for asset in internal_assets.iter() {
        if let Some(generate_source_map) =
            Vc::try_resolve_sidecast::<Box<dyn GenerateSourceMap>>(*asset).await?
        {
            if let Some(path) = intermediate_output_path.get_path_to(&*asset.ident().path().await?)
            {
                internal_assets_for_source_mapping.insert(path.to_string(), generate_source_map);
            }
        }
    }
    Ok(Vc::cell(internal_assets_for_source_mapping))
}

/// Splits the asset graph into "internal" assets and boundaries to "external"
/// assets.
#[turbo_tasks::function]
async fn separate_assets(
    intermediate_asset: Vc<Box<dyn OutputAsset>>,
    intermediate_output_path: Vc<FileSystemPath>,
) -> Result<Vc<SeparatedAssets>> {
    let intermediate_output_path = &*intermediate_output_path.await?;
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    enum Type {
        Internal(Vc<Box<dyn OutputAsset>>),
        External(Vc<Box<dyn OutputAsset>>),
    }
    let get_asset_children = |asset| async move {
        let Type::Internal(asset) = asset else {
            return Ok(Vec::new());
        };
        asset
            .references()
            .await?
            .iter()
            .map(|asset| async {
                // Assets within the output directory are considered as "internal" and all
                // others as "external". We follow references on "internal" assets, but do not
                // look into references of "external" assets, since there are no "internal"
                // assets behind "externals"
                if asset
                    .ident()
                    .path()
                    .await?
                    .is_inside_ref(intermediate_output_path)
                {
                    Ok(Type::Internal(*asset))
                } else {
                    Ok(Type::External(*asset))
                }
            })
            .try_join()
            .await
    };

    let graph = AdjacencyMap::new()
        .skip_duplicates()
        .visit(once(Type::Internal(intermediate_asset)), get_asset_children)
        .await
        .completed()?
        .into_inner();

    let mut internal_assets = IndexSet::new();
    let mut external_asset_entrypoints = IndexSet::new();

    for item in graph.into_reverse_topological() {
        match item {
            Type::Internal(asset) => {
                internal_assets.insert(asset);
            }
            Type::External(asset) => {
                external_asset_entrypoints.insert(asset);
            }
        }
    }

    Ok(SeparatedAssets {
        internal_assets: Vc::cell(internal_assets),
        external_asset_entrypoints: Vc::cell(external_asset_entrypoints),
    }
    .cell())
}

/// Emit a basic package.json that sets the type of the package to commonjs.
/// Currently code generated for Node is CommonJS, while authored code may be
/// ESM, for example.
fn emit_package_json(dir: Vc<FileSystemPath>) -> Vc<Completion> {
    emit(
        Vc::upcast(VirtualOutputAsset::new(
            dir.join("package.json".to_string().into()),
            AssetContent::file(File::from("{\"type\":\"commonjs\"}").into()),
        )),
        dir,
    )
}

pub fn register() {
    turbo_tasks::register();
    turbopack_binding::turbo::tasks_fs::register();
    turbopack_binding::turbo::tasks_bytes::register();
    turbopack_binding::turbopack::ecmascript::register();
    include!(concat!(env!("OUT_DIR"), "/register.rs"));
}
