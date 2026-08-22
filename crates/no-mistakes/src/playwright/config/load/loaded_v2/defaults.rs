use std::path::{Path, PathBuf};

/// Legacy zero-signal fallback: no `type: nextjs` project is configured and
/// none could be inferred at all, so there is no [`FrontendApp`] to derive a
/// route root from. Retained unchanged (still only probes `<nextjs_root>/app`,
/// not `<nextjs_root>/src/app`) because this deepest fallback tier is not the
/// path #625 reported — that case always has a resolvable [`FrontendApp`],
/// which already applies the `src/app`-preferred probe.
pub(super) fn default_frontend_root(
    root: &Path,
    nextjs_root: &str,
    visible_paths: &[PathBuf],
) -> String {
    let app_root = Path::new(nextjs_root).join("app");
    let absolute_app_root = crate::codebase::ts_resolver::normalize_path(&root.join(&app_root));
    if visible_paths.iter().any(|path| {
        crate::codebase::ts_resolver::normalize_path(path).starts_with(&absolute_app_root)
    }) {
        app_root.to_string_lossy().into_owned()
    } else {
        nextjs_root.to_string()
    }
}

pub(super) fn default_selector_test_excludes() -> &'static [&'static str] {
    &[
        "**/*.{test,spec}.{ts,tsx,js,jsx,mts,cts}",
        "**/*.test.*",
        "**/*.spec.*",
    ]
}
