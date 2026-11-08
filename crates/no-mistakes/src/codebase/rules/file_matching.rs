use crate::codebase::ts_source::relative_slash_path;
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::{Path, PathBuf};

pub(crate) struct FileMatcher<'a> {
    root: &'a Path,
    target_roots: &'a [PathBuf],
    globs: GlobSet,
}

impl<'a> FileMatcher<'a> {
    pub(crate) fn new<'p>(
        root: &'a Path,
        target_roots: &'a [PathBuf],
        patterns: impl Iterator<Item = &'p str>,
    ) -> Result<Self> {
        let mut builder = GlobSetBuilder::new();
        for pattern in patterns {
            builder.add(Glob::new(pattern.trim_start_matches("./"))?);
        }
        Ok(Self {
            root,
            target_roots,
            globs: builder.build()?,
        })
    }

    pub(crate) fn is_match(&self, path: &Path) -> bool {
        self.globs.is_match(relative_slash_path(self.root, path))
            || self
                .target_roots
                .iter()
                .filter(|target_root| *target_root != self.root && path.starts_with(target_root))
                .any(|target_root| self.globs.is_match(relative_slash_path(target_root, path)))
    }

    pub(crate) fn select(&self, files: &[PathBuf]) -> Vec<PathBuf> {
        if self.globs.is_empty() {
            return Vec::new();
        }
        files
            .iter()
            .filter(|path| self.is_match(path))
            .cloned()
            .collect()
    }
}

pub(crate) fn matching_files(
    root: &Path,
    patterns: &[String],
    files: &[PathBuf],
    target_roots: &[PathBuf],
) -> Result<Vec<PathBuf>> {
    if patterns.is_empty() {
        return Ok(Vec::new());
    }
    Ok(FileMatcher::new(root, target_roots, patterns.iter().map(String::as_str))?.select(files))
}

#[cfg(test)]
mod tests;
