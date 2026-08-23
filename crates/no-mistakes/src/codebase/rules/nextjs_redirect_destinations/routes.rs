use std::collections::BTreeSet;
use std::path::{Component, Path};

const PAGE_EXTS: &[&str] = &["tsx", "ts", "jsx", "js"];

/// Build App Router route paths from `page.*` files under `app_root`.
///
/// `_`-prefixed segments mark the whole route private. `(group)` and `@slot`
/// segments unwrap. Remaining segments join as `/a/b`; none yields `/`.
pub(super) fn build_route_set(files: &[std::path::PathBuf], app_root: &Path) -> BTreeSet<String> {
    let app_root = crate::codebase::ts_resolver::normalize_path(app_root);
    let mut routes = BTreeSet::new();
    for file in files {
        let file = crate::codebase::ts_resolver::normalize_path(file);
        if !is_page_file(&file) {
            continue;
        }
        let Ok(relative) = file.strip_prefix(&app_root) else {
            continue;
        };
        if let Some(route) = route_from_page_relative(relative) {
            routes.insert(route);
        }
    }
    routes
}

pub(super) fn is_page_file(path: &Path) -> bool {
    let stem = path.file_stem().and_then(|stem| stem.to_str()) == Some("page");
    let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    stem && PAGE_EXTS.contains(&ext)
}

pub(super) fn route_from_page_relative(relative: &Path) -> Option<String> {
    if !is_page_file(relative) {
        return None;
    }
    let mut segments = Vec::new();
    for component in relative
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .components()
    {
        let Component::Normal(segment) = component else {
            continue;
        };
        let segment = segment.to_str()?;
        if segment.starts_with('_') {
            return None;
        }
        if segment.starts_with('@') || (segment.starts_with('(') && segment.ends_with(')')) {
            continue;
        }
        segments.push(segment);
    }
    Some(if segments.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", segments.join("/"))
    })
}

pub(super) fn strip_query_and_hash(destination: &str) -> &str {
    let without_query = destination
        .split_once('?')
        .map_or(destination, |(path, _)| path);
    without_query
        .split_once('#')
        .map_or(without_query, |(path, _)| path)
}

pub(super) fn should_skip_destination(dest_path: &str) -> bool {
    dest_path.contains("://")
        || dest_path.starts_with("//")
        || dest_path
            .as_bytes()
            .windows(2)
            .any(|pair| pair[0] == b':' && pair[1].is_ascii_alphabetic())
}

pub(super) fn destination_matches(route_set: &BTreeSet<String>, dest_path: &str) -> bool {
    if route_set.contains(dest_path) {
        return true;
    }
    let dest_segs = path_segments(dest_path);
    route_set.iter().any(|route| {
        let route_segs = if route == "/" {
            Vec::new()
        } else {
            path_segments(route)
        };
        matches_route_segments(&route_segs, &dest_segs)
    })
}

pub(super) fn matches_route_segments(route_segs: &[&str], dest_segs: &[&str]) -> bool {
    let last_seg = route_segs.last().copied().unwrap_or("");
    if !last_seg.contains("...") {
        return route_segs.len() == dest_segs.len()
            && route_segs
                .iter()
                .zip(dest_segs.iter())
                .all(|(route, dest)| route.starts_with('[') || route == dest);
    }

    let prefix_len = route_segs.len().saturating_sub(1);
    let min_extra = if last_seg.starts_with("[[") { 0 } else { 1 };
    let prefix_segs = &route_segs[..prefix_len];
    dest_segs.len() >= prefix_len + min_extra
        && prefix_segs
            .iter()
            .zip(dest_segs.iter())
            .all(|(route, dest)| route.starts_with('[') || route == dest)
}

fn path_segments(path: &str) -> Vec<&str> {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .collect()
}
