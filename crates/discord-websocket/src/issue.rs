use turbopack_binding::{
    turbo::{
        tasks as turbo_tasks,
        tasks::{RcStr, Vc},
        tasks_fs::FileSystemPath,
    },
    turbopack::core::issue::{Issue, IssueSeverity, IssueStage, OptionStyledString, StyledString},
};

/// An issue that occurred when processing events/commands from the user.
#[turbo_tasks::value(shared)]
pub struct RuntimeIssue {
    pub path: Vc<FileSystemPath>,
    pub severity: Option<Vc<IssueSeverity>>,
    pub title: RcStr,
    pub description: Option<RcStr>,
}

#[turbo_tasks::value_impl]
impl Issue for RuntimeIssue {
    #[turbo_tasks::function]
    fn title(&self) -> Vc<StyledString> {
        StyledString::Text(self.title.clone()).cell()
    }

    #[turbo_tasks::function]
    fn severity(&self) -> Vc<IssueSeverity> {
        self.severity.unwrap_or(IssueSeverity::Fatal.cell())
    }

    #[turbo_tasks::function]
    fn stage(&self) -> Vc<IssueStage> {
        IssueStage::Other("runtime".to_string()).cell()
    }

    #[turbo_tasks::function]
    fn file_path(&self) -> Vc<FileSystemPath> {
        self.path
    }

    #[turbo_tasks::function]
    fn description(&self) -> Vc<OptionStyledString> {
        Vc::cell(
            self.description
                .as_ref()
                .map(|string| StyledString::Text(string.clone()).cell()),
        )
    }
}
