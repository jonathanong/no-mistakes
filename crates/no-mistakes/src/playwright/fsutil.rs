use anyhow::{Context, Result};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub(crate) fn build_globset(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = GlobBuilder::new(pattern).literal_separator(false).build()?;
        builder.add(glob);
    }
    Ok(builder.build()?)
}

/// Collect every file under `root`, pruning directories matched by
/// [`is_skipped_dir`] at any depth.
///
/// Prefers the git-visible file list (`git ls-files` plus untracked-but-not-
/// ignored files) when `root` is inside a git repository: the raw `WalkDir`
/// fallback below has no `.gitignore` awareness beyond `is_skipped_dir`'s
/// small hardcoded denylist, so on real repos it can descend into large
/// untracked-and-ignored directories (dependency stores, build caches) that
/// `git ls-files` would never surface. The raw walk still runs, unchanged,
/// outside git repositories (e.g. ad-hoc test fixtures).
pub(crate) fn walk_files(root: &Path) -> Vec<PathBuf> {
    let mut files = match no_mistakes::codebase::ts_source::git_visible_files(root) {
        Some(git_files) => git_visible_matching_files(root, &git_files),
        None => walk_files_raw(root),
    };
    files.sort();
    files
}

fn git_visible_matching_files(root: &Path, git_files: &[String]) -> Vec<PathBuf> {
    git_files
        .iter()
        .filter(|rel| !is_under_skipped_dir(Path::new(rel)))
        .map(|rel| root.join(rel))
        // Mirrors `WalkDir`'s default (non-follow-symlink) `file_type().is_file()`
        // check: a symlink to a file is not itself a file entry.
        .filter(|path| std::fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_file()))
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

fn walk_files_raw(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_entry(|entry| !(entry.file_type().is_dir() && is_skipped_dir(entry.path())))
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .collect()
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
    slash_path(path.strip_prefix(root).unwrap_or(path))
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
