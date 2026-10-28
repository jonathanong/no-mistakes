//! Call sources a Vitest root executes: its calls plus module scope of everything it imports.
use crate::codebase::dependencies::graph::{CallRoot, CallTraversal, DepGraph, EdgeKind, NodeId};
use crate::fx::FxHashSet;
use std::collections::HashSet;
use std::path::PathBuf;

pub(super) fn sources(
    graph: &DepGraph,
    roots: &[NodeId],
    root_files: &[PathBuf],
) -> FxHashSet<NodeId> {
    let entries: Vec<NodeId> = root_files.iter().map(NodeId::file).collect();
    // Importing a module runs its top level, so each imported module's
    // module-scope calls start a call chain of their own.
    let loads: HashSet<EdgeKind> = [
        EdgeKind::Import,
        EdgeKind::WorkspaceImport,
        EdgeKind::Require,
        EdgeKind::DynamicImport,
        EdgeKind::ConditionalDynamicImport,
    ]
    .into();
    let modules: Vec<CallRoot> = graph
        .deps_of(&entries, None, Some(&loads))
        .into_iter()
        .filter_map(|entry| {
            entry
                .node
                .as_file()
                .map(|file| CallRoot::Module(file.to_path_buf()))
        })
        .collect();
    let mut all = roots.to_vec();
    all.extend(graph.expand_call_roots(&modules));
    all.sort();
    all.dedup();
    graph.call_source_nodes(&all, CallTraversal::Transitive, None)
}
