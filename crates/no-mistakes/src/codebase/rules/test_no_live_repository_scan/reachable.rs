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
    setups: &super::owned::setups::ConfiguredSetups,
) -> anyhow::Result<Scope> {
    let setup_files: Vec<PathBuf> = setups
        .setup_files()
        .unwrap_or_default()
        .into_iter()
        .collect();
    let config_roots = setups
        .config_files()
        .map(|files| files.into_iter().collect::<Vec<_>>());
    let required: Vec<NodeId> = root_files
        .iter()
        .chain(&setup_files)
        .chain(config_roots.iter().flatten())
        .map(NodeId::file)
        .collect();
    super::super::graph_source_prerequisites::require_import_sources(
        super::RULE_ID,
        graph,
        &required,
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
    let setup_modules = module_files(graph, &setup_files);
    let mut all = roots.to_vec();
    all.extend(graph.expand_call_roots(&modules));
    all.extend(
        graph.expand_call_roots(
            &setup_modules
                .iter()
                .cloned()
                .map(CallRoot::Module)
                .collect::<Vec<_>>(),
        ),
    );
    all.sort();
    all.dedup();
    let sources = graph.call_source_nodes(&all, CallTraversal::Transitive, None);
    let mut gate_roots = root_files.to_vec();
    gate_roots.extend(setup_files.iter().cloned());
    if let Some(globals) = setups.global_setup_files() {
        gate_roots.extend(globals);
    }
    let test_closure = module_files(graph, &gate_roots);
    let mut files: FxHashSet<PathBuf> = root_files.iter().cloned().collect();
    files.extend(test_modules);
    files.extend(setup_modules);
    files.extend(
        sources
            .iter()
            .filter_map(NodeId::as_file)
            .map(std::path::Path::to_path_buf),
    );
    let config_files = config_roots
        .as_ref()
        .map(|configs| module_files(graph, configs));
    if let Some(configs) = &config_files {
        files.extend(configs.iter().cloned());
    } else {
        files.extend(graph_files.iter().cloned());
    }
    Ok(Scope {
        sources,
        files,
        config_files,
        test_closure,
        gate_roots,
    })
}

pub(super) struct Scope {
    pub(super) sources: FxHashSet<NodeId>,
    pub(super) files: FxHashSet<PathBuf>,
    pub(super) config_files: Option<FxHashSet<PathBuf>>,
    /// Test files, `setupFiles`, `globalSetup`, and their import closure.
    pub(super) test_closure: FxHashSet<PathBuf>,
    /// Those roots before `test_closure` follows their imports.
    pub(super) gate_roots: Vec<PathBuf>,
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

pub(super) fn load_kinds() -> HashSet<EdgeKind> {
    [
        EdgeKind::Import,
        EdgeKind::WorkspaceImport,
        EdgeKind::Require,
        EdgeKind::DynamicImport,
        EdgeKind::ConditionalDynamicImport,
    ]
    .into()
}
