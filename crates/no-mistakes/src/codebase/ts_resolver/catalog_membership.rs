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
                    .map(|path| membership_path(path))
                    .filter(|path| membership_owns_source(config, path))
                    .collect();
                (config.path.clone(), members)
            })
            .collect()
    }

    /// Coverage needs the union, rather than a retained set for every config.
    pub(crate) fn project_source_union(
        root: &Path,
        config_paths: &[PathBuf],
        visible_paths: &[PathBuf],
        sources: &crate::codebase::ts_source::SourceStore,
        workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    ) -> BTreeSet<PathBuf> {
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
        // References can add catalog entries, but only requested configs count.
        let configs = catalog
            .configs
            .iter()
            .filter(|config| requested.contains(&config.path))
            .collect::<Vec<_>>();
        if configs.is_empty() {
            return BTreeSet::new();
        }
        visible_paths
            .iter()
            .map(|path| membership_path(path))
            .filter(|path| {
                configs
                    .iter()
                    .any(|config| membership_owns_source(config, path))
            })
            .collect()
    }
}

fn membership_path(path: &Path) -> PathBuf {
    #[cfg(any(test, feature = "test-instrumentation"))]
    catalog_membership_work::record_normalization();
    normalize_path(path)
}

fn membership_owns_source(config: &CatalogConfig, path: &Path) -> bool {
    #[cfg(any(test, feature = "test-instrumentation"))]
    catalog_membership_work::record_check();
    config.matcher.owns_source(path)
}
