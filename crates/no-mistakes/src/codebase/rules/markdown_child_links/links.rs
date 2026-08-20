use crate::codebase::md_links;
use crate::codebase::ts_source::FrozenPathRemapper;
use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

#[derive(Debug)]
pub(super) struct ResolvedLink {
    pub(super) path: PathBuf,
    pub(super) whole_file: bool,
}

pub(super) fn resolve_parent_links(
    root: &Path,
    source: &Path,
    destinations: &[String],
    known: &BTreeSet<PathBuf>,
    remapper: &FrozenPathRemapper,
) -> Vec<ResolvedLink> {
    let mut links = Vec::new();
    for destination in destinations {
        let (path_part, whole_file) = split_destination(destination);
        if path_part.is_empty() || md_links::is_external(path_part) {
            continue;
        }
        let Some(decoded) = md_links::decode_local_path(path_part) else {
            continue;
        };
        let base = if decoded.starts_with('/') {
            root.to_path_buf()
        } else {
            source.parent().unwrap_or(root).to_path_buf()
        };
        let Some(path) = normalize_inside(root, &base.join(decoded.trim_start_matches('/'))) else {
            continue;
        };
        if let Some(path) = remapper.remap(&path).filter(|path| known.contains(path)) {
            links.push(ResolvedLink { path, whole_file });
        }
    }
    links
}

fn split_destination(destination: &str) -> (&str, bool) {
    let path_part = destination.split(['#', '?']).next().unwrap_or_default();
    (path_part, !destination.contains('#'))
}

pub(super) fn normalize_inside(root: &Path, path: &Path) -> Option<PathBuf> {
    let mut relative = PathBuf::new();
    for component in path.strip_prefix(root).ok()?.components() {
        match component {
            Component::Normal(part) => relative.push(part),
            Component::ParentDir if relative.pop() => {}
            _ => return None,
        }
    }
    Some(root.join(relative))
}
