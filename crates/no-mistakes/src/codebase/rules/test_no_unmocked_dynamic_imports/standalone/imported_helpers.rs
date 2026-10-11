use super::{reachable, resolve_mock_specifiers};
use crate::codebase::dependencies::graph::DepGraph;
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
    super::super::setup_helper_graph::collect(graph, test_file, visible_files, excluded, |file| {
        let facts = reachable::get_or_cache_file(&file.to_path_buf(), Some(file_cache))?;
        Ok(resolve_mock_specifiers(
            &facts.mock_specifiers,
            file,
            resolver,
            None,
        ))
    })
}

pub(super) fn collect_setup(
    graph: &DepGraph,
    setup: &Path,
    visible_files: &crate::fx::PathSet,
    resolver: &dyn ImportResolution,
    file_cache: &DashMap<PathBuf, Arc<reachable::CachedFileFacts>>,
    excluded: &HashSet<PathBuf>,
) -> Result<HashSet<PathBuf>> {
    super::super::setup_helper_graph::collect(graph, setup, visible_files, excluded, |file| {
        let facts = reachable::get_or_cache_file(&file.to_path_buf(), Some(file_cache))?;
        Ok(resolve_mock_specifiers(
            &facts.mock_specifiers,
            file,
            resolver,
            None,
        ))
    })
}
