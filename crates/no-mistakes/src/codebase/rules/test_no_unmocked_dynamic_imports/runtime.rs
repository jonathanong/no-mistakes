use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, NodeId};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(crate) fn runtime_deps(
    graph: &DepGraph,
    target: PathBuf,
    file_universe: Option<&crate::fx::PathSet>,
) -> Vec<PathBuf> {
    runtime_deps_with_cuts(graph, target, file_universe, None)
}

pub(crate) fn runtime_deps_excluding_mocks(
    graph: &DepGraph,
    target: PathBuf,
    file_universe: Option<&crate::fx::PathSet>,
    mocks: &HashSet<PathBuf>,
) -> Vec<PathBuf> {
    runtime_deps_with_cuts(graph, target, file_universe, Some(mocks))
}

fn runtime_deps_with_cuts(
    graph: &DepGraph,
    target: PathBuf,
    file_universe: Option<&crate::fx::PathSet>,
    mocks: Option<&HashSet<PathBuf>>,
) -> Vec<PathBuf> {
    let allowed = [
        EdgeKind::Import,
        EdgeKind::DynamicImport,
        EdgeKind::Require,
        EdgeKind::WorkspaceImport,
    ]
    .into();
    let roots = [NodeId::file(target)];
    let entries = match (file_universe, mocks) {
        (Some(universe), Some(mocks)) => {
            graph.deps_of_in_file_universe_excluding_files(&roots, Some(&allowed), universe, mocks)
        }
        (None, Some(mocks)) => graph.deps_of_excluding_files(&roots, Some(&allowed), mocks),
        (Some(universe), None) => {
            graph.deps_of_in_file_universe(&roots, None, Some(&allowed), universe)
        }
        (None, None) => graph.deps_of(&roots, None, Some(&allowed)),
    };
    entries
        .into_iter()
        .filter_map(|entry| entry.node.as_file().map(Path::to_path_buf))
        .collect()
}
