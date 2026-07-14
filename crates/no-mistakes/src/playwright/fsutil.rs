use anyhow::{Context, Result};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use std::path::{Path, PathBuf};

pub(crate) use crate::codebase::ts_source::VisiblePathSnapshot;

pub(crate) fn build_globset(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = GlobBuilder::new(pattern).literal_separator(false).build()?;
        builder.add(glob);
    }
    Ok(builder.build()?)
}

/// Collect visible files under `root`, applying Playwright's hardcoded
/// directory and symlink policies to the shared ignore-aware candidate list.
pub(crate) fn walk_files_from_snapshot(
    root: &Path,
    snapshot: &VisiblePathSnapshot,
) -> Vec<PathBuf> {
    let visible_paths = snapshot.paths_for(root);
    let mut files = visible_matching_files(root, &visible_paths);
    files.sort();
    files
}

fn visible_matching_files(root: &Path, files: &[PathBuf]) -> Vec<PathBuf> {
    let normalized_root = crate::codebase::ts_resolver::normalize_path(root);
    files
        .iter()
        .filter(|path| {
            crate::codebase::ts_resolver::normalize_path(path)
                .strip_prefix(&normalized_root)
                .is_ok_and(|rel| !is_under_skipped_dir(rel))
        })
        // Mirrors `WalkDir`'s default (non-follow-symlink) `file_type().is_file()`
        // check: a symlink to a file is not itself a file entry.
        .filter(|path| std::fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_file()))
        .cloned()
        .collect()
}

/// True when any directory component of `rel` (a path relative to the walk
/// root) matches [`is_skipped_dir`], mirroring `WalkDir`'s `filter_entry`
/// pruning every entry beneath a skipped directory.
fn is_under_skipped_dir(rel: &Path) -> bool {
    rel.parent().is_some_and(|parent| {
        parent
            .components()
            .any(|component| is_skipped_dir(Path::new(component.as_os_str())))
    })
}

pub(crate) fn is_skipped_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            matches!(
                name,
                ".git" | "node_modules" | "target" | "dist" | "build" | "coverage" | "test-results"
            )
        })
}

pub(crate) fn relative_string(root: &Path, path: &Path) -> String {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let path = crate::codebase::ts_resolver::normalize_path(path);
    slash_path(path.strip_prefix(root).unwrap_or(&path))
}

pub(crate) fn slash_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub(crate) fn absolutize(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        let cwd =
            std::env::current_dir().context("current working directory must be accessible")?;
        Ok(cwd.join(path))
    }
}
