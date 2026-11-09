//! Call sources a Vitest root executes: its calls plus module scope of everything it imports.
use crate::codebase::dependencies::graph::{CallRoot, CallTraversal, DepGraph, EdgeKind, NodeId};
use crate::fx::FxHashSet;
use std::collections::HashSet;
use std::path::PathBuf;

pub(super) fn sources(
    graph: &DepGraph,
    roots: &[NodeId],
    root_files: &[PathBuf],
    graph_files: &[PathBuf],
) -> anyhow::Result<Scope> {
    super::super::graph_source_prerequisites::require_import_sources(
        super::RULE_ID,
        graph,
        &root_files.iter().map(NodeId::file).collect::<Vec<_>>(),
        None,
    )?;
    let entries: Vec<NodeId> = root_files.iter().map(NodeId::file).collect();
    // Importing a module runs its top level, so each imported module's
    // module-scope calls start a call chain of their own.
    let loads = load_kinds();
    let test_modules: Vec<PathBuf> = graph
        .deps_of(&entries, None, Some(&loads))
        .into_iter()
        .filter_map(|entry| entry.node.as_file().map(std::path::Path::to_path_buf))
        .collect();
    let modules: Vec<CallRoot> = test_modules.iter().cloned().map(CallRoot::Module).collect();
    let mut all = roots.to_vec();
    all.extend(graph.expand_call_roots(&modules));
    all.sort();
    all.dedup();
    let sources = graph.call_source_nodes(&all, CallTraversal::Transitive, None);
    let mut files: FxHashSet<PathBuf> = root_files.iter().cloned().collect();
    files.extend(test_modules);
    files.extend(
        sources
            .iter()
            .filter_map(NodeId::as_file)
            .map(std::path::Path::to_path_buf),
    );
    let configs: Vec<PathBuf> = graph_files
        .iter()
        .filter(|file| super::owned::env::runner_config(file))
        .cloned()
        .collect();
    files.extend(module_files(graph, &configs));
    Ok(Scope { sources, files })
}

pub(super) struct Scope {
    pub(super) sources: FxHashSet<NodeId>,
    pub(super) files: FxHashSet<PathBuf>,
}

pub(super) fn module_files(graph: &DepGraph, roots: &[PathBuf]) -> FxHashSet<PathBuf> {
    let mut files: FxHashSet<PathBuf> = roots.iter().cloned().collect();
    let entries: Vec<NodeId> = roots.iter().map(NodeId::file).collect();
    files.extend(
        graph
            .deps_of(&entries, None, Some(&load_kinds()))
            .into_iter()
            .filter_map(|entry| entry.node.as_file().map(std::path::Path::to_path_buf)),
    );
    files
}

fn load_kinds() -> HashSet<EdgeKind> {
    [
        EdgeKind::Import,
        EdgeKind::WorkspaceImport,
        EdgeKind::Require,
        EdgeKind::DynamicImport,
        EdgeKind::ConditionalDynamicImport,
    ]
    .into()
}
