// TODO(kijv) Add individual sources for commands and events (like
// turbopack-dev-server)
use std::collections::HashMap;

use anyhow::Result;
use fujinoki_core::structures::{
    commands::{CommandsMetadata, OptionCommandsDir},
    events::{EventsMetadata, OptionEventsDir},
};
use turbopack_binding::{
    turbo::{
        tasks as turbo_tasks,
        tasks::{Completion, RcStr, Vc},
        tasks_env::ProcessEnv,
        tasks_fs::FileSystemPath,
    },
    turbopack::core::{
        chunk::{ChunkingContext, EvaluatableAssets},
        context::AssetContext,
        module::Module,
    },
};

/// A source of content that the dev server uses to respond to http requests.
#[turbo_tasks::value_trait]
pub trait ContentSource {
    fn get_entries(self: Vc<Self>) -> Vc<EntryMap>;

    fn get_events(self: Vc<Self>) -> Vc<EventsMetadata> {
        EventsMetadata::default().cell()
    }

    fn get_events_dir(self: Vc<Self>) -> Vc<OptionEventsDir> {
        OptionEventsDir::default().cell()
    }

    fn get_commands(self: Vc<Self>) -> Vc<CommandsMetadata> {
        CommandsMetadata::default().cell()
    }

    fn get_commands_dir(self: Vc<Self>) -> Vc<OptionCommandsDir> {
        OptionCommandsDir::default().cell()
    }

    /// Gets any content sources wrapped in this content source.
    fn get_children(self: Vc<Self>) -> Vc<ContentSources> {
        ContentSources::empty()
    }
}

pub trait ContentSourceExt: Send {
    fn issue_file_path(
        self: Vc<Self>,
        file_path: Vc<FileSystemPath>,
        description: String,
    ) -> Vc<Box<dyn ContentSource>>;
}

#[turbo_tasks::value(shared, serialization = "auto_for_input")]
#[derive(Clone, Debug, Hash)]
pub struct ContentSourceData {
    pub project_path: Vc<FileSystemPath>,
    pub executor: Vc<Executor>,
    pub entries: Vc<EntryMap>,
    pub events: Vc<EventsMetadata>,
    pub commands: Vc<CommandsMetadata>,
    pub events_dir: Vc<OptionEventsDir>,
    pub commands_dir: Vc<OptionCommandsDir>,
}

#[turbo_tasks::value(shared, serialization = "auto_for_input")]
#[derive(Clone, Debug, Hash, Default)]
pub struct OptionalContentSourceData {
    pub project_path: Option<Vc<FileSystemPath>>,
    pub executor: Option<Vc<Executor>>,
    pub entries: Option<Vc<EntryMap>>,
}

#[turbo_tasks::value_impl]
impl ContentSource for ContentSourceData {
    #[turbo_tasks::function]
    fn get_entries(&self) -> Vc<EntryMap> {
        self.entries
    }

    #[turbo_tasks::function]
    fn get_events(&self) -> Vc<EventsMetadata> {
        self.events
    }

    #[turbo_tasks::function]
    fn get_commands(&self) -> Vc<CommandsMetadata> {
        self.commands
    }

    #[turbo_tasks::function]
    fn get_events_dir(&self) -> Vc<OptionEventsDir> {
        self.events_dir
    }

    #[turbo_tasks::function]
    fn get_commands_dir(&self) -> Vc<OptionCommandsDir> {
        self.commands_dir
    }
}

#[turbo_tasks::value(transparent)]
pub struct ContentSources(Vec<Vc<Box<dyn ContentSource>>>);

#[turbo_tasks::value_impl]
impl ContentSources {
    #[turbo_tasks::function]
    pub fn empty() -> Vc<Self> {
        Vc::cell(Vec::new())
    }
}

/// An empty ContentSource implementation that responds with NotFound for every
/// request.
#[turbo_tasks::value]
pub struct NoContentSource;

#[turbo_tasks::value_impl]
impl NoContentSource {
    #[turbo_tasks::function]
    pub fn new() -> Vc<Self> {
        NoContentSource.cell()
    }
}
#[turbo_tasks::value_impl]
impl ContentSource for NoContentSource {
    #[turbo_tasks::function]
    fn get_entries(&self) -> Vc<EntryMap> {
        EntryMap::empty().cell()
    }
}

/// This trait can be emitted as collectible and will be applied after the
/// request is handled and it's ensured that it finishes before the next request
/// is handled.
#[turbo_tasks::value_trait]
pub trait ContentSourceSideEffect {
    fn apply(self: Vc<Self>) -> Vc<Completion>;
}

#[turbo_tasks::value(shared)]
#[derive(Clone, Default, Debug)]
pub struct EntryMap {
    map: HashMap<RcStr, Vc<Box<dyn Module>>>,
}

impl EntryMap {
    /// Creates a new entry map.
    pub fn new(map: HashMap<RcStr, Vc<Box<dyn Module>>>) -> EntryMap {
        Self { map }
    }

    /// Creates a new empty entry map.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Inserts a entry into the entry map.
    pub fn insert_entry(&mut self, path: RcStr, entry: Vc<Box<dyn Module>>) {
        self.map.insert(path, entry);
    }
}

#[turbo_tasks::value_impl]
impl EntryMap {
    #[turbo_tasks::function]
    pub async fn get_entry(&self, path: Vc<RcStr>) -> Result<Vc<Option<Vc<Box<dyn Module>>>>> {
        Ok(Vc::cell(self.map.get(&*path.await?).copied()))
    }
}

#[derive(Debug)]
#[turbo_tasks::value]
pub struct Executor {
    pub cwd: Vc<FileSystemPath>,
    pub env: Vc<Box<dyn ProcessEnv>>,
    pub asset_context: Vc<Box<dyn AssetContext>>,
    pub chunking_context: Vc<Box<dyn ChunkingContext>>,
    pub runtime_entries: Option<Vc<EvaluatableAssets>>,
}

#[turbo_tasks::value_impl]
impl Executor {
    #[turbo_tasks::function]
    pub fn new(
        cwd: Vc<FileSystemPath>,
        env: Vc<Box<dyn ProcessEnv>>,
        asset_context: Vc<Box<dyn AssetContext>>,
        chunking_context: Vc<Box<dyn ChunkingContext>>,
        runtime_entries: Option<Vc<EvaluatableAssets>>,
    ) -> Vc<Self> {
        Executor {
            cwd,
            env,
            asset_context,
            chunking_context,
            runtime_entries,
        }
        .cell()
    }
}
