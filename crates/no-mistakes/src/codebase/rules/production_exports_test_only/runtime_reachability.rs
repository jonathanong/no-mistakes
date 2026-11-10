use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, NodeId};
use crate::fx::FxHashSet;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) fn is_runtime_resolved_unknown(reason: &str) -> bool {
    reason.starts_with("Runtime-resolved import")
}

/// Files production can load, plus whether an unresolved import in that set
/// might load something else.
pub(super) struct ProductionReachability {
    reachable: FxHashSet<PathBuf>,
    open: bool,
}

impl ProductionReachability {
    pub(super) fn collect(
        graph: &DepGraph,
        facts: &CheckFactMap,
        production: impl Fn(&Path) -> bool,
        incomplete: &[(PathBuf, usize, String)],
    ) -> Self {
        let mut reachable = FxHashSet::default();
        let mut roots = Vec::new();
        for file in facts.graph_file_universe() {
            if production(file) {
                reachable.insert(file.clone());
                roots.push(NodeId::file(file));
            }
        }
        // Resolved loads only. A computed import has no target edge; `open`
        // keeps every other runtime unknown instead of guessing its target.
        let allowed: HashSet<EdgeKind> = [
            EdgeKind::Import,
            EdgeKind::DynamicImport,
            EdgeKind::ConditionalDynamicImport,
            EdgeKind::Require,
            EdgeKind::WorkspaceImport,
            EdgeKind::SymbolImportUnknown,
        ]
        .into();
        for entry in graph.deps_of(&roots, None, Some(&allowed)) {
            if let Some(path) = entry.node.as_file() {
                reachable.insert(path.to_path_buf());
            }
        }
        let open = incomplete.iter().any(|(file, _, reason)| {
            is_runtime_resolved_unknown(reason) && reachable.contains(file)
        });
        Self { reachable, open }
    }

    pub(super) fn reports(&self, file: &Path) -> bool {
        self.open || self.reachable.contains(file)
    }
}
