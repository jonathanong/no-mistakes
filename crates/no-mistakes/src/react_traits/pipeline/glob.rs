use anyhow::Result;
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use std::path::{Path, PathBuf};

const EXTENSIONS: &[&str] = &["tsx", "ts", "jsx", "js"];

pub(crate) fn expand_globs_from_files(
    root: &Path,
    patterns: &[String],
    visible_paths: &[PathBuf],
) -> Result<Vec<PathBuf>> {
    if patterns.is_empty() {
        return Ok(Vec::new());
    }
    // VisiblePathSnapshot normalizes its returned candidates. Use the same
    // lexical form for prefix and glob matching without canonicalizing away a
    // caller-selected symlink root.
    let root = crate::codebase::ts_source::normalize_discovery_path(root);
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = GlobBuilder::new(pattern).literal_separator(false).build()?;
        builder.add(glob);
    }
    let globset = builder.build()?;

    // Apply React-specific glob, extension, and skip-directory rules to the
    // caller's ignore-aware candidate list.
    let mut files = visible_matching_files(&root, visible_paths, &globset);
    files.sort();
    Ok(files)
}

fn visible_matching_files(root: &Path, files: &[PathBuf], globset: &GlobSet) -> Vec<PathBuf> {
    files
        .iter()
        .filter_map(|path| {
            let path = crate::codebase::ts_source::normalize_discovery_path(path);
            let matches = path.strip_prefix(root).is_ok_and(|rel| {
                !is_under_skip_dir(rel) && has_matching_extension(rel) && globset.is_match(rel)
            });
            matches.then_some(path)
        })
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

fn is_skip_dir(path: &Path) -> bool {
    path.file_name().and_then(|n| n.to_str()).is_some_and(|n| {
        n.starts_with('.') || matches!(n, "node_modules" | "target" | "dist" | "build" | "coverage")
    })
}

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
