use super::{reachable, resolve_mock_specifiers};
use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, NodeId};
use crate::codebase::ts_resolver::ImportResolution;
use anyhow::Result;
use dashmap::DashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) fn collect(
    graph: &DepGraph,
    test_file: &Path,
    visible_files: &crate::fx::PathSet,
    resolver: &dyn ImportResolution,
    file_cache: &DashMap<PathBuf, Arc<reachable::CachedFileFacts>>,
    excluded: &HashSet<PathBuf>,
) -> Result<HashSet<PathBuf>> {
    let allowed = [EdgeKind::Import, EdgeKind::WorkspaceImport].into();
    let mut mocks = HashSet::new();
    for entry in graph.deps_of_in_file_universe(
        &[NodeId::file(test_file)],
        None,
        Some(&allowed),
        visible_files,
    ) {
        let Some(file) = entry.node.as_file() else {
            continue;
        };
        if excluded.contains(file) || !crate::codebase::dependencies::extract::is_indexable(file) {
            continue;
        }
        let facts = reachable::get_or_cache_file(&file.to_path_buf(), Some(file_cache))?;
        mocks.extend(resolve_mock_specifiers(
            &facts.mock_specifiers,
            file,
            resolver,
            None,
        ));
    }
    Ok(mocks)
}
