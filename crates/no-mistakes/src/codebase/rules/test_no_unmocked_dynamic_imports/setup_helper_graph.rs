use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, NodeId};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// A helper's mock may prevent another helper from executing. Graph adjacency
/// is path-ordered, not source-ordered, so conservatively prune every helper
/// mocked by a potentially reachable helper before counting registrations.
pub(super) fn collect(
    graph: &DepGraph,
    setup: &Path,
    visible_files: &crate::fx::PathSet,
    initial_mocks: &HashSet<PathBuf>,
    mut registrations: impl FnMut(&Path) -> Result<HashSet<PathBuf>>,
) -> Result<HashSet<PathBuf>> {
    let allowed = [EdgeKind::Import, EdgeKind::WorkspaceImport].into();
    let helper_files = |excluded: &HashSet<PathBuf>| {
        graph
            .deps_of_in_file_universe_excluding_files(
                &[NodeId::file(setup)],
                Some(&allowed),
                visible_files,
                excluded,
            )
            .into_iter()
            .filter_map(|entry| entry.node.as_file().map(Path::to_path_buf))
            .filter(|file| {
                !excluded.contains(file)
                    && crate::codebase::dependencies::extract::is_indexable(file)
            })
            .collect::<Vec<_>>()
    };

    let mut potential_cuts = initial_mocks.clone();
    let mut by_helper = HashMap::new();
    for file in helper_files(initial_mocks) {
        let outcome = registrations(&file);
        if let Ok(mocks) = &outcome {
            potential_cuts.extend(mocks.iter().cloned());
        }
        by_helper.insert(file, outcome);
    }

    let mut active_mocks = initial_mocks.clone();
    for file in helper_files(&potential_cuts) {
        if let Some(outcome) = by_helper.remove(&file) {
            active_mocks.extend(outcome?);
        }
    }
    Ok(active_mocks)
}
