use crate::codebase::workspaces;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub(super) fn load_workspace_with_sources(
    root: &Path,
    target_roots: &[PathBuf],
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<workspaces::WorkspaceMap> {
    if let Some(observer) = sources.observer() {
        observer.increment("forbidden_workspace_closure.workspace_selections", 1);
    }
    let roots = std::iter::once(root).chain(target_roots.iter().map(PathBuf::as_path));
    let inputs = workspaces::load_manual_workspace_from_source_store(roots, files, sources)?;
    Ok(inputs.workspace)
}
