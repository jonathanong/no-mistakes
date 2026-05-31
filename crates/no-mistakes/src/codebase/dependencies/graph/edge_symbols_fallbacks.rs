fn fallback_imported_symbols<'a>(
    include_all: bool,
    calls: &[crate::codebase::dependencies::extract::FunctionCall],
    refs: &[crate::codebase::dependencies::extract::FunctionCall],
    imported_symbols: &'a HashMap<String, ImportedSymbolTarget>,
) -> Vec<&'a ImportedSymbolTarget> {
    let mut imports = Vec::new();
    if include_all {
        imports.extend(imported_symbols.values());
        imports.sort_by_key(|target| target_node(target));
        imports.dedup_by_key(|target| target_node(target));
        return imports;
    }
    for call in calls.iter().chain(refs) {
        if call.caller.is_some() {
            continue;
        }
        if let Some(target) = imported_symbols.get(&call.callee) {
            imports.push(target);
        }
    }
    imports.sort_by_key(|target| target_node(target));
    imports.dedup_by_key(|target| target_node(target));
    imports
}

fn fallback_namespace_symbols(
    calls: &[crate::codebase::dependencies::extract::FunctionCall],
    refs: &[crate::codebase::dependencies::extract::FunctionCall],
    namespace_imports: &HashMap<String, ImportedSymbolTarget>,
) -> Vec<(NodeId, EdgeKind)> {
    let mut nodes = Vec::new();
    for call in calls.iter().chain(refs) {
        if call.caller.is_some() {
            continue;
        }
        let Some((namespace, member)) = call.callee.split_once('.') else {
            continue;
        };
        let Some(target) = namespace_imports.get(namespace) else {
            continue;
        };
        nodes.push(namespace_target_node(target, member));
    }
    nodes.sort();
    nodes.dedup();
    nodes
}

fn collect_top_level_imported_edges(
    path: &Path,
    caller_to_export: &HashMap<String, Vec<String>>,
    calls: &[crate::codebase::dependencies::extract::FunctionCall],
    imported_symbols: &HashMap<String, ImportedSymbolTarget>,
    edges: &mut Vec<Edge>,
) {
    let exports = exported_symbol_names(caller_to_export);
    if exports.is_empty() {
        return;
    }
    for imported in fallback_imported_symbols(false, calls, &[], imported_symbols) {
        let (target, kind) = target_node(imported);
        for export in &exports {
            edges.push((
                NodeId::Symbol {
                    file: path.to_path_buf(),
                    symbol: export.clone(),
                },
                target.clone(),
                kind,
            ));
        }
    }
}

fn exported_symbol_names(caller_to_export: &HashMap<String, Vec<String>>) -> Vec<String> {
    let mut exports: Vec<_> = caller_to_export
        .values()
        .flat_map(|exports| exports.iter().cloned())
        .collect();
    exports.sort();
    exports.dedup();
    exports
}
