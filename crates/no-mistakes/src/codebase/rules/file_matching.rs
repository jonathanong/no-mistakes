use crate::codebase::ts_source::relative_slash_path;
use anyhow::Result;
use globset::{Glob, GlobSetBuilder};
use std::path::{Path, PathBuf};

pub(crate) fn matching_files(
    root: &Path,
    patterns: &[String],
    files: &[PathBuf],
    target_roots: &[PathBuf],
) -> Result<Vec<PathBuf>> {
    if patterns.is_empty() {
        return Ok(Vec::new());
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern.trim_start_matches("./"))?);
    }
    let globs = builder.build()?;
    Ok(files
        .iter()
        .filter(|path| {
            globs.is_match(relative_slash_path(root, path))
                || target_roots
                    .iter()
                    .filter(|target_root| *target_root != root && path.starts_with(target_root))
                    .any(|target_root| globs.is_match(relative_slash_path(target_root, path)))
        })
        .cloned()
        .collect())
}

#[cfg(test)]
mod tests;
