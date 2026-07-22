use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, NodeId};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(crate) fn runtime_deps(
    graph: &DepGraph,
    target: PathBuf,
    file_universe: Option<&HashSet<PathBuf>>,
) -> Vec<PathBuf> {
    let allowed = [
        EdgeKind::Import,
        EdgeKind::DynamicImport,
        EdgeKind::Require,
        EdgeKind::WorkspaceImport,
    ]
    .into();
    let roots = [NodeId::File(target)];
    let entries = match file_universe {
        Some(universe) => graph.deps_of_in_file_universe(&roots, None, Some(&allowed), universe),
        None => graph.deps_of(&roots, None, Some(&allowed)),
    };
    entries
        .into_iter()
        .filter_map(|entry| entry.node.as_file().map(Path::to_path_buf))
        .collect()
}
