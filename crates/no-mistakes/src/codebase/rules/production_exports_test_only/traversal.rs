use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, NodeId};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_symbols::ExportKind;
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
        let (line, message) = module_gap_message(ts, module);
        if seen.insert((line, message.clone())) {
            incomplete.push((file.to_path_buf(), line, message));
        }
    }
}

fn module_gap_message(ts: &TsFileFacts, module: &str) -> (usize, String) {
    if let Some((resolved, symbol)) = module
        .rsplit_once('#')
        .filter(|(path, _)| Path::new(path).is_absolute())
    {
        let (specifier, line) = import_site(ts, symbol, true).unwrap_or((resolved.to_owned(), 1));
        return (
            line,
            format!(
                "unresolved symbol `{symbol}` from `{specifier}` prevents complete per-symbol analysis"
            ),
        );
    }
    (
        import_site(ts, module, false).map_or(1, |(_, line)| line),
        format!("unresolved module selector `{module}` prevents complete per-symbol analysis"),
    )
}

fn import_site(ts: &TsFileFacts, key: &str, symbol: bool) -> Option<(String, usize)> {
    if let Some(symbols) = ts.symbols.as_ref() {
        if let Some(import) = symbols.imports.iter().find(|import| {
            if symbol {
                import.imported == key
            } else {
                import.source == key
            }
        }) {
            return Some((import.source.clone(), import.line.max(1) as usize));
        }
        if let Some((source, line)) = symbols.exports.iter().find_map(|export| {
            let ExportKind::ReExport { source, imported } = &export.kind else {
                return None;
            };
            let matched = if symbol {
                imported == key
            } else {
                source == key
            };
            matched.then(|| (source.clone(), export.line.max(1) as usize))
        }) {
            return Some((source, line));
        }
    }
    if symbol {
        return None;
    }
    if let Some(import) = ts.imports.iter().find(|import| import.specifier == key) {
        return Some((key.to_owned(), import.line.max(1) as usize));
    }
    let import = ts
        .module_bindings
        .as_ref()?
        .imports
        .iter()
        .find(|import| import.specifier == key)?;
    Some((
        key.to_owned(),
        crate::codebase::ts_source::byte_offset_to_line(
            ts.source.as_deref()?,
            import.span.start as usize,
        )
        .max(1) as usize,
    ))
}
