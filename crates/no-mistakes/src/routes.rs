use crate::codebase::ts_source::discover_visible_paths;
use std::path::{Path, PathBuf};

const PAGE_EXTS: &[&str] = &["tsx", "ts", "jsx", "js"];

pub struct Route {
    pub file: PathBuf,
    pub pattern: String,
}

/// Collect `Route`s under `frontend_root` whose file stem is in `stems` and whose
/// extension is one of `PAGE_EXTS`.
///
/// Candidates come from shared Git-visible discovery, with `.gitignore` still
/// applied outside Git repositories. Route-specific dot-directory filtering is
/// then applied in memory.
pub fn collect_routes(frontend_root: &Path, stems: &[&str]) -> Vec<Route> {
    if !frontend_root.exists() {
        return Vec::new();
    }

    let files = discover_visible_paths(frontend_root);
    let mut routes = collect_routes_from_visible_paths(frontend_root, &files, stems);

    routes.sort_by(|a, b| a.pattern.cmp(&b.pattern).then_with(|| a.file.cmp(&b.file)));
    routes
}

#[doc(hidden)]
pub fn collect_routes_from_visible_paths(
    frontend_root: &Path,
    files: &[PathBuf],
    stems: &[&str],
) -> Vec<Route> {
    let mut routes = Vec::new();
    for file in files {
        let Ok(relative) = file.strip_prefix(frontend_root) else {
            continue;
        };
        if is_under_dot_directory(relative) || !matches_route_file(relative, stems) {
            continue;
        }
        if !file.is_file() {
            continue;
        }

        routes.push(Route {
            file: file.clone(),
            pattern: path_to_route_pattern(relative),
        });
    }
    routes
}

fn matches_route_file(relative: &Path, stems: &[&str]) -> bool {
    let stem = relative.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let ext = relative.extension().and_then(|e| e.to_str()).unwrap_or("");
    stems.contains(&stem) && PAGE_EXTS.contains(&ext)
}

/// True if any directory component of `relative` starts with `.`, preserving
/// the route collector's historical dot-directory exclusion.
fn is_under_dot_directory(relative: &Path) -> bool {
    relative
        .parent()
        .into_iter()
        .flat_map(Path::components)
        .any(|component| {
            component
                .as_os_str()
                .to_str()
                .is_some_and(|name| name.starts_with('.'))
        })
}

pub fn path_to_route_pattern(relative: &Path) -> String {
    let dir: PathBuf = relative
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_default();

    let mut segments = Vec::new();
    for component in dir.components() {
        let std::path::Component::Normal(segment) = component else {
            continue;
        };
        let segment = segment.to_str().unwrap_or("");

        if segment.starts_with('@') || (segment.starts_with('(') && segment.ends_with(')')) {
            continue;
        }

        if segment.starts_with("[[...") && segment.ends_with("]]") {
            segments.push("**".to_string());
            continue;
        }

        if segment.starts_with("[...") && segment.ends_with(']') {
            segments.push("*".to_string());
            continue;
        }

        if segment.starts_with('[') && segment.ends_with(']') {
            segments.push(format!(":{}", &segment[1..segment.len() - 1]));
            continue;
        }

        segments.push(segment.to_string());
    }

    if segments.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", segments.join("/"))
    }
}

pub mod rewrites;

#[cfg(test)]
mod tests;
