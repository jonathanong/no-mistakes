use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, NodeId};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::{fx_set, FxHashSet};
use std::collections::{BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

#[derive(Default)]
pub(super) struct Consumers {
    pub(super) files: BTreeSet<PathBuf>,
    pub(super) unknown: BTreeSet<(PathBuf, NodeId)>,
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
                    result
                        .unknown
                        .insert((path.to_path_buf(), NodeId::file(file)));
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
    if let Some(observer) = crate::diagnostics::current() {
        observer.increment("production_exports.incomplete_scans", 1);
    }
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
        push_module_selector_gaps(file, ts, graph, &mut incomplete);
    }
    incomplete
}

fn push_module_selector_gaps(
    file: &Path,
    ts: &TsFileFacts,
    graph: &DepGraph,
    incomplete: &mut Vec<(PathBuf, usize, String)>,
) {
    let Some(edges) = graph.dependencies_of_node(&NodeId::file(file)) else {
        return;
    };
    let mut seen = BTreeSet::new();
    for (target, kind) in edges {
        if *kind != EdgeKind::SymbolImportUnknown {
            continue;
        }
        let NodeId::Module(module) = target else {
            continue;
        };
        let sites = graph.unknown_selector_sites(file, target);
        if !sites.is_empty() {
            for site in sites {
                let detail = module
                    .rsplit_once('#')
                    .filter(|(path, _)| Path::new(path).is_absolute())
                    .map_or_else(
                        || format!("unresolved module selector `{}`", site.specifier),
                        |(_, symbol)| {
                            format!("unresolved symbol `{symbol}` from `{}`", site.specifier)
                        },
                    );
                let message = format!("{detail} prevents complete per-symbol analysis");
                if seen.insert((site.line, message.clone())) {
                    incomplete.push((file.to_path_buf(), site.line, message));
                }
            }
            continue;
        }
        let (line, message) = super::selector_gaps::module_gap_message(ts, module);
        if seen.insert((line, message.clone())) {
            incomplete.push((file.to_path_buf(), line, message));
        }
    }
}
