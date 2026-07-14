use crate::routes as core_routes;
use std::path::{Path, PathBuf};

pub use core_routes::Route;

const PAGE_STEMS: &[&str] = &["page"];

pub(crate) fn collect_routes_from_visible(
    frontend_root: &Path,
    visible_paths: &[PathBuf],
) -> Vec<Route> {
    let frontend_root = crate::codebase::ts_resolver::normalize_path(frontend_root);
    let mut routes = visible_paths
        .iter()
        .filter_map(|file| {
            let normalized_file = crate::codebase::ts_resolver::normalize_path(file);
            let relative = normalized_file.strip_prefix(&frontend_root).ok()?;
            if relative
                .parent()
                .into_iter()
                .flat_map(Path::components)
                .any(|component| {
                    component
                        .as_os_str()
                        .to_str()
                        .is_some_and(|name| name.starts_with('.'))
                })
            {
                return None;
            }
            let stem = relative.file_stem().and_then(|stem| stem.to_str())?;
            let extension = relative
                .extension()
                .and_then(|extension| extension.to_str())?;
            if !PAGE_STEMS.contains(&stem)
                || !["tsx", "ts", "jsx", "js"].contains(&extension)
                || !file.is_file()
            {
                return None;
            }
            Some(Route {
                file: file.clone(),
                pattern: core_routes::path_to_route_pattern(relative),
            })
        })
        .collect::<Vec<_>>();
    routes.sort_by(|a, b| a.pattern.cmp(&b.pattern).then_with(|| a.file.cmp(&b.file)));
    routes
}
