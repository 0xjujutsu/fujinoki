use anyhow::Result;
use turbopack_binding::{
    turbo::{
        tasks as turbo_tasks,
        tasks::{RcStr, Vc},
        tasks_fs::{FileSystem, FileSystemPath, VirtualFileSystem},
    },
    turbopack::core::issue::{Issue, IssueSeverity, IssueStage, OptionStyledString, StyledString},
};

#[turbo_tasks::value(shared)]
pub struct DiscordApiIssue {
    pub severity: Vc<IssueSeverity>,
    pub file_path: Option<Vc<FileSystemPath>>,
    pub message: Vc<StyledString>,
    pub title: Option<RcStr>,
}

#[turbo_tasks::value_impl]
impl Issue for DiscordApiIssue {
    #[turbo_tasks::function]
    fn severity(&self) -> Vc<IssueSeverity> {
        self.severity
    }

    #[turbo_tasks::function]
    async fn title(&self) -> Result<Vc<StyledString>> {
        Ok(StyledString::Text(
            self.title.clone().unwrap_or(
                "An issue occurred while accessing the Discord API"
                    .to_string()
                    .into(),
            ),
        )
        .cell())
    }

    #[turbo_tasks::function]
    fn stage(&self) -> Vc<IssueStage> {
        IssueStage::Other("runtime".to_string()).cell()
    }

    #[turbo_tasks::function]
    fn file_path(&self) -> Vc<FileSystemPath> {
        self.file_path.unwrap_or(VirtualFileSystem::new().root())
    }

    #[turbo_tasks::function]
    fn description(&self) -> Vc<OptionStyledString> {
        Vc::cell(Some(self.message))
    }
}
