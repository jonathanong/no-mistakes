use super::*;

pub(super) fn catalog(
    root: &Path,
    tsconfig: &TsConfig,
    tsconfig_path: Option<&Path>,
    roots: &[PathBuf],
    visible: &[PathBuf],
    sources: &Arc<SourceStore>,
) -> Arc<TsConfigCatalog> {
    Arc::new(if let Some(path) = tsconfig_path {
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        };
        TsConfigCatalog::forced(
            root,
            tsconfig.clone(),
            Some(no_mistakes::codebase::ts_resolver::normalize_path(&path)),
        )
    } else {
        TsConfigCatalog::from_visible_and_sources(root, roots, visible, sources)
    })
}
