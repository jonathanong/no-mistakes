fn edge_kind_for_import(import: &ExtractedImport) -> EdgeKind {
    match import.kind {
        ImportKind::Static => EdgeKind::Import,
        ImportKind::Type => EdgeKind::TypeImport,
        ImportKind::Dynamic => EdgeKind::DynamicImport,
        ImportKind::Require => EdgeKind::Require,
    }
}

fn import_is_reachable(
    import: &ExtractedImport,
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
    reachable: &HashSet<String>,
) -> bool {
    let Some(scope) = &import.function_scope else {
        return true;
    };
    facts.has_unknown_top_level_call
        || reachable.contains(scope)
        || facts.exported_functions.iter().any(|name| name == scope)
}

fn reachable_function_scopes(
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
) -> HashSet<String> {
    let mut by_caller: HashMap<Option<String>, Vec<String>> = HashMap::new();
    for call in &facts.function_calls {
        by_caller
            .entry(call.caller.clone())
            .or_default()
            .push(call.callee.clone());
    }

    let mut reachable = HashSet::new();
    let mut queue: VecDeque<String> = by_caller
        .get(&None)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .collect();
    while let Some(function) = queue.pop_front() {
        if !reachable.insert(function.clone()) {
            continue;
        }
        if let Some(callees) = by_caller.get(&Some(function)) {
            for callee in callees {
                queue.push_back(callee.clone());
            }
        }
    }
    reachable
}

fn bare_module_node(specifier: &str) -> Option<NodeId> {
    if specifier.starts_with('.') || specifier.starts_with('/') || specifier.starts_with('#') {
        return None;
    }
    Some(NodeId::Module(specifier.to_string()))
}
