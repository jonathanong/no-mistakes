pub(crate) fn load_root_package_from_visible_sources(
    dir: &Path,
    visible: &dyn VisiblePathLookup,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Option<WorkspacePackage>> {
    load_package_with_sources(dir, Some(visible), WorkspaceSources::Store(sources))
}
