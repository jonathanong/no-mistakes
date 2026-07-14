use super::super::{PlaywrightFactPlan, PlaywrightModuleResolution};
use std::path::Path;
use std::sync::Arc;

pub(super) fn initialize_if_missing(
    root: &Path,
    playwright: &mut PlaywrightFactPlan,
    sources: &crate::codebase::ts_source::SourceStore,
) {
    if playwright.module_resolution().is_some() {
        return;
    }
    let visible_paths = sources.inventory().paths();
    let Ok(tsconfig) = crate::codebase::ts_resolver::resolve_tsconfig_from_visible_and_sources(
        None,
        root,
        &visible_paths,
        sources,
    ) else {
        return;
    };
    let visible_files = Arc::new(
        visible_paths
            .iter()
            .map(|path| crate::codebase::ts_resolver::normalize_path(path))
            .collect(),
    );
    let workspace = Arc::new(
        crate::codebase::workspaces::load_indexed_from_source_store(root, sources)
            .unwrap_or_default(),
    );
    playwright.set_module_resolution(Arc::new(PlaywrightModuleResolution::new(
        Arc::new(tsconfig),
        workspace,
        visible_files,
    )));
}
