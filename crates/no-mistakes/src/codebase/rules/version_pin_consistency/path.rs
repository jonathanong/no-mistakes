use crate::codebase::ts_source::is_portably_absolute_path;
use std::path::{Component, Path, PathBuf};

pub(super) fn read_text(
    root: &Path,
    rel: &str,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<std::sync::Arc<str>, String> {
    let Some(path) = contained_regular_file(root, rel) else {
        return Err(format!(
            "{rel} is missing or is not a readable repository file"
        ));
    };
    sources
        .read_path(&path)
        .map_err(|error| format!("cannot read {rel}: {error}"))
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
