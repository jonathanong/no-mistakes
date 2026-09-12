use crate::codebase::ts_source::is_portably_absolute_path;
use std::path::{Component, Path, PathBuf};

pub(super) fn read_text(
    root: &Path,
    rel: &str,
    sources: &crate::codebase::ts_source::SourceStore,
) -> String {
    let Some(path) = contained_regular_file(root, rel) else {
        return String::new();
    };
    super::super::read_source(sources, &path)
        .map(|source| source.to_string())
        .unwrap_or_else(|| std::fs::read_to_string(&path).unwrap_or_default())
}

fn contained_regular_file(root: &Path, rel: &str) -> Option<PathBuf> {
    if !repo_relative(rel) {
        return None;
    }
    let path = root.join(rel);
    let metadata = std::fs::symlink_metadata(&path).ok()?;
    if !metadata.is_file() {
        return None;
    }
    let canonical_root = std::fs::canonicalize(root).ok()?;
    let canonical = std::fs::canonicalize(&path).ok()?;
    canonical.starts_with(&canonical_root).then_some(path)
}

fn repo_relative(rel: &str) -> bool {
    let path = Path::new(rel);
    !is_portably_absolute_path(path)
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}
