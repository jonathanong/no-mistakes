pub struct SharedWorkspace {
    pub map: Option<Arc<no_mistakes::codebase::workspaces::IndexedWorkspaceMap>>,
    pub deadlines: bool,
}

