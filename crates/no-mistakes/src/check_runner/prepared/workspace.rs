pub(crate) struct SharedWorkspace {
    pub(crate) map: Option<Arc<no_mistakes::codebase::workspaces::IndexedWorkspaceMap>>,
    pub(crate) deadlines: bool,
}

