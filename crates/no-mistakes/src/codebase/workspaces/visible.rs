/// Normalize the request's visibility once before loading multiple packages.
pub(crate) fn normalized_visible_files(files: &[PathBuf]) -> crate::fx::PathSet {
    files
        .iter()
        .map(|path| {
            record_preparation!(visible_path);
            normalize_path(path)
        })
        .collect()
}

pub(crate) fn load_root_package_from_visible_sources(
    dir: &Path,
    visible: &dyn VisiblePathLookup,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Option<WorkspacePackage>> {
    load_package_with_sources(dir, Some(visible), WorkspaceSources::Store(sources))
}
