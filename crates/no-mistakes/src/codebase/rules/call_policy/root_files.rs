use super::super::path_filter::GlobMatcher;
use crate::codebase::dependencies::graph::DepGraph;
use crate::codebase::ts_source::relative_slash_path;
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

pub(super) fn excludes_root(root: &Path, file: &Path, exclude_roots: &GlobMatcher) -> bool {
    exclude_roots.is_match(&relative_slash_path(root, file))
}

pub(super) fn reject_parse_error(
    rule: &str,
    graph: &DepGraph,
    root: &Path,
    path: &Path,
) -> Result<()> {
    let Some(error) = graph.parse_error(path) else {
        return Ok(());
    };
    bail!(
        "{rule}: configured root `{}` failed to parse: {error}",
        relative_slash_path(root, path)
    );
}

pub(super) fn require_file(
    rule: &str,
    root: &Path,
    configured: &str,
    kind: &str,
) -> Result<PathBuf> {
    let path = Path::new(configured);
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    if !path.is_file() {
        bail!("{rule}: {kind} `{configured}` is not a file");
    }
    Ok(crate::codebase::ts_resolver::normalize_path(&path))
}
