use crate::codebase::ts_source::git_visible_files;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const PAGE_EXTS: &[&str] = &["tsx", "ts", "jsx", "js"];

pub struct Route {
    pub file: PathBuf,
    pub pattern: String,
}

/// Collect `Route`s under `frontend_root` whose file stem is in `stems` and whose
/// extension is one of `PAGE_EXTS`.
///
/// Prefers the git-visible file list (tracked files plus untracked files not
/// excluded by `.gitignore`) when `frontend_root` is inside a git repository:
/// candidates are filtered purely in-memory from that list, no filesystem walk.
/// This matters because the raw-walk fallback below only skips dot-directories,
/// so on real frontend projects with `node_modules`/`dist`/`.next`/etc. under
/// `frontend_root` it can descend into tens of thousands of vendored files just
/// to find files named e.g. `page.tsx`. The raw walk is used only outside git
/// repositories (e.g. ad-hoc test fixtures).
pub fn collect_routes(frontend_root: &Path, stems: &[&str]) -> Vec<Route> {
    if !frontend_root.exists() {
        return Vec::new();
    }

    let mut routes = match git_visible_files(frontend_root) {
        Some(files) => collect_routes_from_git_files(frontend_root, &files, stems),
        None => collect_routes_by_walk(frontend_root, stems),
    };

    routes.sort_by(|a, b| a.pattern.cmp(&b.pattern).then_with(|| a.file.cmp(&b.file)));
    routes
}

fn collect_routes_from_git_files(
    frontend_root: &Path,
    files: &[String],
    stems: &[&str],
) -> Vec<Route> {
    let mut routes = Vec::new();
    for rel in files {
        let relative = Path::new(rel);
        if is_under_dot_directory(relative) || !matches_route_file(relative, stems) {
            continue;
        }

        routes.push(Route {
            file: frontend_root.join(relative),
            pattern: path_to_route_pattern(relative),
        });
    }
    routes
}

fn collect_routes_by_walk(frontend_root: &Path, stems: &[&str]) -> Vec<Route> {
    let mut routes = Vec::new();
    for entry in WalkDir::new(frontend_root)
        .into_iter()
        .filter_entry(|e| {
            !(e.file_type().is_dir() && e.file_name().to_str().is_some_and(|n| n.starts_with('.')))
        })
        .filter_map(|entry| entry.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if let Ok(relative) = path.strip_prefix(frontend_root) {
            if matches_route_file(relative, stems) {
                routes.push(Route {
                    file: path.to_path_buf(),
                    pattern: path_to_route_pattern(relative),
                });
            }
        }
    }
    routes
}

fn matches_route_file(relative: &Path, stems: &[&str]) -> bool {
    let stem = relative.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let ext = relative.extension().and_then(|e| e.to_str()).unwrap_or("");
    stems.contains(&stem) && PAGE_EXTS.contains(&ext)
}

/// Mirrors the raw walk's dot-directory `filter_entry` skip: true if any directory
/// component of `relative` (i.e. every component except the final file name) starts
/// with `.`. Needed because `git ls-files` can surface tracked files under
/// dot-directories (e.g. a committed `.next/` build-output fixture) that the raw walk
/// would never have descended into.
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
