use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, NodeId};
use crate::fx::{fx_set, FxHashSet};
use std::collections::{BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

#[derive(Default)]
pub(super) struct Consumers {
    pub(super) files: BTreeSet<PathBuf>,
    pub(super) unknown: BTreeSet<PathBuf>,
}

pub(super) fn consumers(graph: &DepGraph, file: &Path, name: &str) -> Consumers {
    let start = NodeId::symbol(file, name);
    let mut visited: FxHashSet<NodeId> = fx_set();
    let mut queue = VecDeque::from([start.clone()]);
    visited.insert(start);
    let mut result = Consumers::default();
    while let Some(node) = queue.pop_front() {
        let file = node
            .as_file()
            .expect("selector relay nodes have source files");
        if let Some(edges) = graph.dependents_of_node(&NodeId::file(file)) {
            for (importer, kind) in edges {
                if *kind == EdgeKind::SymbolImportUnknown {
                    let path = importer
                        .as_file()
                        .expect("unknown selector importers are files");
                    result.unknown.insert(path.to_path_buf());
                }
            }
        }
        let Some(edges) = graph.dependents_of_node(&node) else {
            continue;
        };
        for (importer, kind) in edges {
            match kind {
                EdgeKind::SymbolImport => {
                    let path = importer.as_file().expect("selector importers are files");
                    result.files.insert(path.to_path_buf());
                }
                EdgeKind::SymbolReexport if visited.insert(importer.clone()) => {
                    queue.push_back(importer.clone());
                }
                _ => {}
            }
        }
    }
    result
}

/// Unbounded unknown targets can hide a production consumer of any export.
pub(super) fn incomplete_sources(
    graph: &DepGraph,
    facts: &CheckFactMap,
) -> Vec<(PathBuf, usize, String)> {
    let mut incomplete = Vec::new();
    for file in facts.graph_file_universe() {
        if !crate::codebase::dependencies::extract::is_indexable(file) {
            continue;
        }
        let Some(ts) = facts.get_ts_facts(file) else {
            incomplete.push((
                file.clone(),
                1,
                "missing prepared importer facts".to_owned(),
            ));
            continue;
        };
        if ts.parse_error.is_some() || ts.operational_error.is_some() || ts.symbols.is_none() {
            incomplete.push((
                file.clone(),
                1,
                "importer exports or imports could not be analyzed".to_owned(),
            ));
            continue;
        }
        if let Some(bindings) = &ts.module_bindings {
            for diagnostic in &bindings.diagnostics {
                let line = diagnostic.span.as_ref().zip(ts.source.as_deref()).map_or(
                    1,
                    |(span, source)| {
                        crate::codebase::ts_source::byte_offset_to_line(source, span.start as usize)
                            as usize
                    },
                );
                incomplete.push((file.clone(), line, diagnostic.message.clone()));
            }
        }
        for import in ts.imports.iter().filter(|import| import.computed) {
            incomplete.push((file.clone(), import.line as usize,
                "computed import target prevents complete per-symbol analysis; use a literal named import".to_owned()));
        }
        if graph
            .dependencies_of_node(&NodeId::file(file))
            .is_some_and(|edges| {
                edges.iter().any(|(target, kind)| {
                    *kind == EdgeKind::SymbolImportUnknown && target.as_file().is_none()
                })
            })
        {
            incomplete.push((
                file.clone(),
                1,
                "unresolved module selector prevents complete per-symbol analysis".to_owned(),
            ));
        }
    }
    incomplete
}
