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

fn value_export_symbol_names(
    symbols: &crate::codebase::ts_symbols::FileSymbols,
) -> HashSet<String> {
    symbols
        .exports
        .iter()
        .filter(|export| !export.is_type_only)
        .map(export_symbol_name)
        .collect()
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
    resolver: &dyn ImportResolution,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    visible_files: &HashSet<PathBuf>,
    graph_files: &GraphFiles,
) -> HashMap<String, ImportedSymbolTarget> {
    let mut map = HashMap::new();
    for import in &symbols.imports {
        if import.imported == "*" {
            continue;
        }
        let kind = symbol_edge_kind(import.is_type_only);
        let target = if let Some(target) = resolver.resolve(&import.source, path) {
            let Some(target) = graph_files.visible_path(&target) else {
                continue;
            };
            if is_indexable(target) {
                ImportedSymbolTarget::Symbol {
                    file: target.to_path_buf(),
                    symbol: import.imported.clone(),
                    kind,
                }
            } else {
                ImportedSymbolTarget::Node {
                    node: NodeId::File(target.to_path_buf()),
                    kind: EdgeKind::AssetImport,
                }
            }
        } else if let Some(target) = workspace.resolve_specifier_from_file_visible(
            &import.source,
            path,
            visible_files,
        ) {
            if !visible_files.contains(&target) {
                continue;
            }
            ImportedSymbolTarget::Symbol {
                file: target,
                symbol: import.imported.clone(),
                kind: EdgeKind::WorkspaceImport,
            }
        } else if workspace.recognizes_specifier_from(&import.source, path) {
            continue;
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
    resolver: &dyn ImportResolution,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    visible_files: &HashSet<PathBuf>,
    graph_files: &GraphFiles,
) -> HashMap<String, ImportedSymbolTarget> {
    let mut map = HashMap::new();
    for import in &symbols.imports {
        if import.imported != "*" {
            continue;
        }
        let kind = symbol_edge_kind(import.is_type_only);
        let target = if let Some(file) = resolver.resolve(&import.source, path) {
            let Some(file) = graph_files.visible_path(&file) else {
                continue;
            };
            if is_indexable(file) {
                ImportedSymbolTarget::Symbol {
                    file: file.to_path_buf(),
                    symbol: "*".to_string(),
                    kind,
                }
            } else {
                ImportedSymbolTarget::Node {
                    node: NodeId::File(file.to_path_buf()),
                    kind: EdgeKind::AssetImport,
                }
            }
        } else if let Some(file) = workspace.resolve_specifier_from_file_visible(
            &import.source,
            path,
            visible_files,
        ) {
            if !visible_files.contains(&file) {
                continue;
            }
            ImportedSymbolTarget::Symbol {
                file,
                symbol: "*".to_string(),
                kind: EdgeKind::WorkspaceImport,
            }
        } else if workspace.recognizes_specifier_from(&import.source, path) {
            continue;
        } else if let Some(node) = bare_module_node(&import.source) {
            ImportedSymbolTarget::Node { node, kind }
        } else {
            continue;
        };
        map.insert(import.local.clone(), target);
    }
    map
}

include!("edge_symbols_helpers_reexports.rs");

fn target_export_is_type(target: &Path, symbol: &str, facts: &dyn TsFactLookup) -> bool {
    let Some(symbols) = facts
        .get_ts_facts(target)
        .and_then(|facts| facts.symbols.as_ref())
    else {
        return false;
    };
    let mut has_type = false;
    let mut has_value = false;
    for export in &symbols.exports {
        if export_symbol_name(export) != symbol {
            continue;
        }
        if export.is_type_only {
            has_type = true;
        } else {
            has_value = true;
        }
    }
    has_type && !has_value
}

fn symbol_edge_kind(is_type_only: bool) -> EdgeKind {
    if is_type_only {
        EdgeKind::TypeImport
    } else {
        EdgeKind::Import
    }
}
