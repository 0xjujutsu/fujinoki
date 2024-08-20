use turbopack_binding::{
    turbo::{tasks as turbo_tasks, tasks::Vc, tasks_fs::FileSystemPath},
    turbopack::core::issue::{Issue, IssueSeverity, IssueStage, OptionStyledString, StyledString},
};

/// An issue that occurred when loading configuration
#[turbo_tasks::value(shared)]
pub struct ConfigIssue {
    pub path: Vc<FileSystemPath>,
    // TODO refurbish this to RcStr
    pub description: Vc<StyledString>,
}

#[turbo_tasks::value_impl]
impl Issue for ConfigIssue {
    #[turbo_tasks::function]
    fn title(&self) -> Vc<StyledString> {
        StyledString::Text("Failed to load configuration".into()).cell()
    }

    #[turbo_tasks::function]
    fn severity(&self) -> Vc<IssueSeverity> {
        IssueSeverity::Fatal.cell()
    }

    #[turbo_tasks::function]
    fn stage(&self) -> Vc<IssueStage> {
        IssueStage::Load.cell()
    }

    #[turbo_tasks::function]
    fn file_path(&self) -> Vc<FileSystemPath> {
        self.path
    }

    #[turbo_tasks::function]
    fn description(&self) -> Vc<OptionStyledString> {
        Vc::cell(Some(self.description))
    }
}
