use anyhow::Result;
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use std::path::Path;

pub(crate) fn build_globset(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = GlobBuilder::new(pattern).literal_separator(false).build()?;
        builder.add(glob);
    }
    Ok(builder.build()?)
}

pub(crate) fn relative_string(root: &Path, path: &Path) -> String {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let path = crate::codebase::ts_resolver::normalize_path(path);
    slash_path(path.strip_prefix(root).unwrap_or(&path))
}

pub(crate) fn slash_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
