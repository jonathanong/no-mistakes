use anyhow::Result;
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const EXTENSIONS: &[&str] = &["tsx", "ts", "jsx", "js"];

pub(crate) fn expand_globs(root: &Path, patterns: &[String]) -> Result<Vec<PathBuf>> {
    if patterns.is_empty() {
        return Ok(Vec::new());
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = GlobBuilder::new(pattern).literal_separator(false).build()?;
        builder.add(glob);
    }
    let globset = builder.build()?;

    // Prefer the git-visible file list (tracked plus untracked-but-not-ignored
    // files) when `root` is inside a git repository: the raw `WalkDir` fallback
    // below only prunes a small hardcoded denylist (`is_skip_dir`), so on real
    // repos it can descend into large untracked-and-ignored directories
    // (dependency stores, build caches) that `git ls-files` would never
    // surface. The raw walk still runs, unchanged, outside git repositories
    // (e.g. ad-hoc test fixtures).
    let mut files = match no_mistakes::codebase::ts_source::git_visible_files(root) {
        Some(git_files) => git_visible_matching_files(root, &git_files, &globset),
        None => walk_matching_files_raw(root, &globset),
    };
    files.sort();
    Ok(files)
}

fn git_visible_matching_files(
    root: &Path,
    git_files: &[String],
    globset: &GlobSet,
) -> Vec<PathBuf> {
    git_files
        .iter()
        .map(Path::new)
        .filter(|rel| !is_under_skip_dir(rel))
        .filter(|rel| has_matching_extension(rel))
        .filter(|rel| globset.is_match(rel))
        .map(|rel| root.join(rel))
        // Mirrors `WalkDir`'s default (non-follow-symlink) `file_type().is_file()`
        // check: a symlink to a file is not itself a file entry.
        .filter(|path| std::fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_file()))
        .collect()
}

/// True when any directory component of `rel` (a path relative to the walk
/// root) matches [`is_skip_dir`], mirroring `WalkDir`'s `filter_entry`
/// pruning every entry beneath a skipped directory.
fn is_under_skip_dir(rel: &Path) -> bool {
    rel.parent().is_some_and(|parent| {
        parent
            .components()
            .any(|component| is_skip_dir(Path::new(component.as_os_str())))
    })
}

fn has_matching_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| EXTENSIONS.contains(&ext))
}

fn walk_matching_files_raw(root: &Path, globset: &GlobSet) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let walker = WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !(e.file_type().is_dir() && is_skip_dir(e.path())));
    for entry in walker.filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if !has_matching_extension(path) {
            continue;
        }
        let rel = path.strip_prefix(root).unwrap_or(path);
        if globset.is_match(rel) {
            files.push(path.to_path_buf());
        }
    }
    files
}

fn is_skip_dir(path: &Path) -> bool {
    path.file_name().and_then(|n| n.to_str()).is_some_and(|n| {
        n.starts_with('.') || matches!(n, "node_modules" | "target" | "dist" | "build" | "coverage")
    })
}

#[cfg(test)]
mod tests;
