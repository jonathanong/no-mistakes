fn fallback_imported_symbols<'a>(
    include_all: bool,
    calls: &[crate::codebase::dependencies::extract::FunctionCall],
    imported_symbols: &'a HashMap<String, ImportedSymbolTarget>,
) -> Vec<&'a ImportedSymbolTarget> {
    let mut imports = Vec::new();
    if include_all {
        imports.extend(imported_symbols.values());
        imports.sort_by_key(|target| target_node(target));
        imports.dedup_by_key(|target| target_node(target));
        return imports;
    }
    for call in calls {
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
    namespace_imports: &HashMap<String, ImportedSymbolTarget>,
) -> Vec<(NodeId, EdgeKind)> {
    let mut nodes = Vec::new();
    for call in calls {
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
