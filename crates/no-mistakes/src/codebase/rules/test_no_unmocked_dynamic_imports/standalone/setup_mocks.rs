use super::{imported_helpers, resolve_mock_specifiers};
use crate::codebase::dependencies::graph::{DepGraph, GraphFiles};
use crate::codebase::rules::test_no_unmocked_dynamic_imports::{ast, config, reachable};
use crate::codebase::ts_resolver::ImportResolution;
use anyhow::{Context, Result};
use dashmap::DashMap;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) fn precompute_setup_mock_map(
    root: &Path,
    test_files: &[PathBuf],
    setup_data: &[config::ConfigSetupData],
    resolver: &dyn ImportResolution,
    graph_files: Option<&GraphFiles>,
) -> Result<HashMap<PathBuf, HashSet<PathBuf>>> {
    let unique_setups: HashSet<PathBuf> = test_files
        .iter()
        .flat_map(|file| {
            let rel = crate::codebase::ts_source::relative_slash_path(root, file);
            config::setup_files_for_test_precomputed(&rel, setup_data)
        })
        .collect();
    unique_setups
        .into_iter()
        .map(|setup| {
            let source = std::fs::read_to_string(&setup)
                .context(format!("failed to read setup file {}", setup.display()))?;
            let facts = ast::extract(&setup, &source)?;
            Ok((
                setup.clone(),
                resolve_mock_specifiers(&facts.mock_specifiers, &setup, resolver, graph_files),
            ))
        })
        .collect()
}

pub(super) struct SetupMockRequest<'a> {
    pub setup_files: &'a [PathBuf],
    pub mock_map: &'a HashMap<PathBuf, HashSet<PathBuf>>,
    pub graph: &'a DepGraph,
    pub visible_files: &'a crate::fx::PathSet,
    pub resolver: &'a dyn ImportResolution,
    pub file_cache: &'a DashMap<PathBuf, Arc<reachable::CachedFileFacts>>,
}

pub(super) fn setup_mocks(request: SetupMockRequest<'_>) -> Result<HashSet<PathBuf>> {
    let mut mocks = HashSet::new();
    for setup in request.setup_files {
        if let Some(direct) = request.mock_map.get(setup) {
            mocks.extend(direct.iter().cloned());
        }
        mocks.extend(imported_helpers::collect_setup(
            request.graph,
            setup,
            request.visible_files,
            request.resolver,
            request.file_cache,
            &mocks,
        )?);
    }
    Ok(mocks)
}
