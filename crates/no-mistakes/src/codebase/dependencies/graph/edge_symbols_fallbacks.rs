fn fallback_imported_symbols<'a>(
    include_all: bool,
    calls: &[crate::codebase::dependencies::extract::FunctionCall],
    refs: &[crate::codebase::dependencies::extract::FunctionCall],
    imported_symbols: &'a HashMap<String, ImportedSymbolTarget>,
) -> Vec<&'a ImportedSymbolTarget> {
    let mut imports = Vec::new();
    if include_all {
        // ⚡ Bolt: Cache the `target_node` key once per element so it runs exactly once each.
        // `sort_by_cached_key` caches keys for the sort, but `dedup_by_key` would recompute
        // them; pairing keys with targets up front avoids the redundant matching and `NodeId`
        // clones during dedup.
        let mut cached: Vec<_> = imported_symbols
            .values()
            .map(|target| (target_node(target), target))
            .collect();
        cached.sort_by(|a, b| a.0.cmp(&b.0));
        cached.dedup_by(|a, b| a.0 == b.0);
        return cached.into_iter().map(|(_, target)| target).collect();
    }
    for call in calls.iter().chain(refs) {
        if call.caller.is_some() {
            continue;
        }
        if let Some(target) = imported_symbols.get(&call.callee) {
            imports.push(target);
        }
    }
    // ⚡ Bolt: Cache the `target_node` key once per element so it runs exactly once each,
    // avoiding the redundant calls `dedup_by_key` would make after the sort.
    let mut cached: Vec<_> = imports
        .into_iter()
        .map(|target| (target_node(target), target))
        .collect();
    cached.sort_by(|a, b| a.0.cmp(&b.0));
    cached.dedup_by(|a, b| a.0 == b.0);
    cached.into_iter().map(|(_, target)| target).collect()
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
    value_exports: &HashSet<String>,
    edges: &mut Vec<Edge>,
) {
    let mut exports = exported_symbol_names(caller_to_export);
    exports.retain(|export| value_exports.contains(export));
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

fn collect_file_scope_import_edges(
    path: &Path,
    caller_exports: &[String],
    value_exports: &HashSet<String>,
    imports: &[(NodeId, EdgeKind)],
    edges: &mut Vec<Edge>,
) {
    for export_symbol in caller_exports {
        if !value_exports.contains(export_symbol) {
            continue;
        }
        for (target, kind) in imports {
            edges.push((
                NodeId::Symbol {
                    file: path.to_path_buf(),
                    symbol: export_symbol.clone(),
                },
                target.clone(),
                *kind,
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
