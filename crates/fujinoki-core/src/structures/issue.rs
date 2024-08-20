use anyhow::Result;
use turbopack_binding::{
    turbo::{tasks as turbo_tasks, tasks::Vc, tasks_fs::FileSystemPath},
    turbopack::core::issue::{Issue, IssueSeverity, IssueStage, OptionStyledString, StyledString},
};

#[turbo_tasks::value(shared)]
pub struct DirectoryTreeIssue {
    pub severity: Vc<IssueSeverity>,
    pub dir: Vc<FileSystemPath>,
    pub message: Vc<StyledString>,
}

#[turbo_tasks::value_impl]
impl Issue for DirectoryTreeIssue {
    #[turbo_tasks::function]
    fn severity(&self) -> Vc<IssueSeverity> {
        self.severity
    }

    #[turbo_tasks::function]
    async fn title(&self) -> Result<Vc<StyledString>> {
        Ok(StyledString::Text(
            "An issue occurred while preparing your application"
                .to_string()
                .into(),
        )
        .cell())
    }

    #[turbo_tasks::function]
    fn stage(&self) -> Vc<IssueStage> {
        IssueStage::AppStructure.cell()
    }

    #[turbo_tasks::function]
    fn file_path(&self) -> Vc<FileSystemPath> {
        self.dir
    }

    #[turbo_tasks::function]
    fn description(&self) -> Vc<OptionStyledString> {
        Vc::cell(Some(self.message))
    }
}
