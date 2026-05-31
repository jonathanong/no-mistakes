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

fn imported_symbol_map(
    path: &Path,
    symbols: &crate::codebase::ts_symbols::FileSymbols,
    resolver: &ImportResolver<'_>,
) -> HashMap<String, (PathBuf, String, bool)> {
    let mut map = HashMap::new();
    for import in &symbols.imports {
        if import.imported == "*" {
            continue;
        }
        if let Some(target) = resolver.resolve(&import.source, path) {
            map.insert(
                import.local.clone(),
                (target, import.imported.clone(), import.is_type_only),
            );
        }
    }
    map
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
