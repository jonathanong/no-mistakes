impl TsConfigCatalog {
    /// Project each explicitly requested tsconfig onto the visible source files
    /// selected by its `files`/`include`/`exclude` matcher.
    pub(crate) fn project_source_membership(
        root: &Path,
        config_paths: &[PathBuf],
        visible_paths: &[PathBuf],
        sources: &crate::codebase::ts_source::SourceStore,
        workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    ) -> BTreeMap<PathBuf, BTreeSet<PathBuf>> {
        let requested = config_paths
            .iter()
            .map(|path| normalize_path(path))
            .collect::<BTreeSet<_>>();
        let catalog = Self::from_visible_and_sources_with_workspace(
            root,
            config_paths,
            visible_paths,
            sources,
            workspace,
        );
        catalog
            .configs
            .iter()
            .filter(|config| requested.contains(&config.path))
            .map(|config| {
                let members = visible_paths
                    .iter()
                    .map(|path| normalize_path(path))
                    .filter(|path| config.matcher.owns_source(path))
                    .collect();
                (config.path.clone(), members)
            })
            .collect()
    }
}
