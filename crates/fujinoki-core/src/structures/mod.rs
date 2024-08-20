use std::collections::BTreeMap;

use anyhow::Result;
use tracing::Instrument;
use turbopack_binding::turbo::{
    tasks as turbo_tasks,
    tasks::{Completion, Completions, RcStr, ValueToString, Vc},
    tasks_fs::{DirectoryContent, DirectoryEntry, FileSystemPath},
};

pub mod commands;
pub mod events;
pub mod issue;

/// A final route in the `x` directory.
#[turbo_tasks::value(shared)]
#[derive(Default, Debug, Clone)]
pub struct Components(pub BTreeMap<RcStr, Vc<FileSystemPath>>);

#[turbo_tasks::value_impl]
impl Components {
    /// Returns a completion that changes when any route in the components
    /// changes.
    #[turbo_tasks::function]
    pub async fn routes_changed(self: Vc<Self>) -> Result<Vc<Completion>> {
        self.await?;
        Ok(Completion::new())
    }
}

#[turbo_tasks::value(shared)]
#[derive(Clone, Debug)]
pub struct DirectoryTree {
    /// key is e.g. "dashboard", "(dashboard)", "@slot"
    pub subdirectories: BTreeMap<RcStr, Vc<DirectoryTree>>,
    pub components: Vc<Components>,
}

#[turbo_tasks::value_impl]
impl DirectoryTree {
    /// Returns a completion that changes when any route in the whole tree
    /// changes.
    #[turbo_tasks::function]
    pub async fn routes_changed(self: Vc<Self>) -> Result<Vc<Completion>> {
        let DirectoryTree {
            subdirectories,
            components,
        } = &*self.await?;
        let mut children = Vec::new();
        children.push(components.routes_changed());
        for child in subdirectories.values() {
            children.push(child.routes_changed());
        }
        Ok(Vc::<Completions>::cell(children).completed())
    }
}

#[turbo_tasks::function]
pub async fn get_directory_tree(
    dir: Vc<FileSystemPath>,
    file_extensions: Vc<Vec<RcStr>>,
) -> Result<Vc<DirectoryTree>> {
    let span = {
        let dir = dir.to_string().await?.to_string();
        tracing::info_span!("read events directory tree", name = dir)
    };
    get_directory_tree_internal(dir, file_extensions)
        .instrument(span)
        .await
}

async fn get_directory_tree_internal(
    dir: Vc<FileSystemPath>,
    file_extensions: Vc<Vec<RcStr>>,
) -> Result<Vc<DirectoryTree>> {
    let DirectoryContent::Entries(entries) = &*dir.read_dir().await? else {
        // the file watcher might invalidate things in the wrong order,
        // and we have to account for the eventual consistency of turbo-tasks
        // so we just return an empty tree here.
        return Ok(DirectoryTree {
            subdirectories: Default::default(),
            components: Components::default().cell(),
        }
        .cell());
    };
    let file_extensions_value = file_extensions.await?;

    let mut subdirectories = BTreeMap::new();
    let mut components = Components::default();

    for (basename, entry) in entries {
        match *entry {
            DirectoryEntry::File(file) => {
                let file = file.resolve().await?;
                // Do not process .d.ts files as routes
                if basename.ends_with(".d.ts") {
                    continue;
                }
                if let Some((stem, ext)) = basename.split_once('.') {
                    if file_extensions_value.iter().any(|e| e == ext) {
                        components.0.insert(stem.to_string().into(), file);
                    }
                }
            }
            DirectoryEntry::Directory(dir) => {
                let dir = dir.resolve().await?;
                // appDir ignores paths starting with an underscore
                if !basename.starts_with('_') {
                    let result = get_directory_tree(dir, file_extensions);
                    subdirectories.insert(basename.clone(), result);
                }
            }
            // TODO(turbopack) handle symlinks in app dir
            _ => {}
        }
    }

    Ok(DirectoryTree {
        subdirectories,
        components: components.cell(),
    }
    .cell())
}
