fn export_symbol_name(export: &crate::codebase::ts_symbols::Export) -> String {
    if matches!(export.kind, ExportKind::Default) {
        "default".to_string()
    } else {
        export.name.clone()
    }
}

fn export_local_name(export: &crate::codebase::ts_symbols::Export) -> String {
    export.local.clone().unwrap_or_else(|| export.name.clone())
}

#[derive(Clone)]
enum ImportedSymbolTarget {
    Symbol {
        file: PathBuf,
        symbol: String,
        kind: EdgeKind,
    },
    Node {
        node: NodeId,
        kind: EdgeKind,
    },
}

fn imported_symbol_map(
    path: &Path,
    symbols: &crate::codebase::ts_symbols::FileSymbols,
    resolver: &ImportResolver<'_>,
    workspace: &crate::codebase::workspaces::WorkspaceMap,
) -> HashMap<String, ImportedSymbolTarget> {
    let mut map = HashMap::new();
    for import in &symbols.imports {
        if import.imported == "*" {
            continue;
        }
        let kind = symbol_edge_kind(import.is_type_only);
        let target = if let Some(target) = resolver.resolve(&import.source, path) {
            if is_indexable(&target) {
                ImportedSymbolTarget::Symbol {
                    file: target,
                    symbol: import.imported.clone(),
                    kind,
                }
            } else {
                ImportedSymbolTarget::Node {
                    node: NodeId::File(target),
                    kind: EdgeKind::AssetImport,
                }
            }
        } else if let Some(target) = workspace.resolve_specifier_from(&import.source, path) {
            ImportedSymbolTarget::Symbol {
                file: target,
                symbol: import.imported.clone(),
                kind: EdgeKind::WorkspaceImport,
            }
        } else if let Some(node) = bare_module_node(&import.source) {
            ImportedSymbolTarget::Node { node, kind }
        } else {
            continue;
        };
        map.insert(import.local.clone(), target);
    }
    map
}

fn namespace_import_map(
    path: &Path,
    symbols: &crate::codebase::ts_symbols::FileSymbols,
    resolver: &ImportResolver<'_>,
    workspace: &crate::codebase::workspaces::WorkspaceMap,
) -> HashMap<String, ImportedSymbolTarget> {
    let mut map = HashMap::new();
    for import in &symbols.imports {
        if import.imported != "*" {
            continue;
        }
        let kind = symbol_edge_kind(import.is_type_only);
        let target = if let Some(file) = resolver.resolve(&import.source, path) {
            if is_indexable(&file) {
                ImportedSymbolTarget::Symbol {
                    file,
                    symbol: "*".to_string(),
                    kind,
                }
            } else {
                ImportedSymbolTarget::Node {
                    node: NodeId::File(file),
                    kind: EdgeKind::AssetImport,
                }
            }
        } else if let Some(file) = workspace.resolve_specifier_from(&import.source, path) {
            ImportedSymbolTarget::Symbol {
                file,
                symbol: "*".to_string(),
                kind: EdgeKind::WorkspaceImport,
            }
        } else if let Some(node) = bare_module_node(&import.source) {
            ImportedSymbolTarget::Node { node, kind }
        } else {
            continue;
        };
        map.insert(import.local.clone(), target);
    }
    map
}

fn resolve_imported_callee(
    callee: &str,
    imported_symbols: &HashMap<String, ImportedSymbolTarget>,
    namespace_imports: &HashMap<String, ImportedSymbolTarget>,
    facts: &dyn TsFactLookup,
    resolver: &ImportResolver<'_>,
    workspace: &crate::codebase::workspaces::WorkspaceMap,
) -> Option<(NodeId, EdgeKind)> {
    if let Some(target) = imported_symbols.get(callee) {
        return Some(target_node(target));
    }
    let (namespace, member) = callee.split_once('.')?;
    if let Some(target) = namespace_imports.get(namespace) {
        return Some(namespace_target_node(target, member));
    }
    let ImportedSymbolTarget::Symbol {
        file: barrel,
        symbol: imported,
        kind,
    } = imported_symbols.get(namespace)?
    else {
        return None;
    };
    let barrel_symbols = facts.get_ts_facts(barrel)?.symbols.as_ref()?;
    for export in &barrel_symbols.exports {
        if export.name != *imported {
            continue;
        }
        let ExportKind::ReExport {
            source,
            imported: reexported,
        } = &export.kind
        else {
            continue;
        };
        if reexported != "*" {
            continue;
        }
        let (target, source_kind) = if let Some(target) = resolver.resolve(source, barrel) {
            (target, *kind)
        } else {
            (
                workspace.resolve_specifier_from(source, barrel)?,
                EdgeKind::WorkspaceImport,
            )
        };
        return Some((
            NodeId::Symbol {
                file: target,
                symbol: member.to_string(),
            },
            if *kind == EdgeKind::TypeImport || export.is_type_only {
                EdgeKind::TypeImport
            } else {
                source_kind
            },
        ));
    }
    None
}

fn target_export_is_type(target: &Path, symbol: &str, facts: &dyn TsFactLookup) -> bool {
    facts
        .get_ts_facts(target)
        .and_then(|facts| facts.symbols.as_ref())
        .and_then(|symbols| {
            symbols
                .exports
                .iter()
                .find(|export| export_symbol_name(export) == symbol)
        })
        .is_some_and(|export| export.is_type_only)
}

fn local_call_graph(
    calls: &[crate::codebase::dependencies::extract::FunctionCall],
) -> HashMap<String, Vec<String>> {
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    for call in calls {
        if let Some(caller) = &call.caller {
            graph
                .entry(caller.clone())
                .or_default()
                .push(call.callee.clone());
        }
    }
    for callees in graph.values_mut() {
        callees.sort();
        callees.dedup();
    }
    graph
}

fn symbol_edge_kind(is_type_only: bool) -> EdgeKind {
    if is_type_only {
        EdgeKind::TypeImport
    } else {
        EdgeKind::Import
    }
}
