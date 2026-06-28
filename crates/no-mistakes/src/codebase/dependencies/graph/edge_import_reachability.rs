fn edge_kind_for_import(import: &ExtractedImport) -> EdgeKind {
    match import.kind {
        ImportKind::Static => EdgeKind::Import,
        ImportKind::Type => EdgeKind::TypeImport,
        ImportKind::Dynamic => EdgeKind::DynamicImport,
        ImportKind::Require | ImportKind::RequireResolve => EdgeKind::Require,
    }
}

fn import_is_reachable(
    import: &ExtractedImport,
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
    reachable: &HashSet<String>,
) -> bool {
    // A runtime `import()`/`require()` collected from inside an exported binding
    // initializer (e.g. `next/dynamic(() => import('./Foo'))`) lives in an
    // anonymous callback scope that no static call reaches, but it is still
    // loaded whenever the exported binding is used. The extractor flags these at
    // collection time, so treat them as reachable here.
    if import.runtime_reachable {
        return true;
    }
    let Some(scope) = &import.function_scope else {
        return true;
    };
    facts.has_unknown_top_level_call
        || has_reachable_unknown_call(facts, reachable)
        || reachable.contains(scope)
        || facts.exported_functions.iter().any(|name| name == scope)
        || (import.kind == ImportKind::Type && exported_symbol_scope(facts, scope))
}

fn has_reachable_unknown_call(
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
    reachable: &HashSet<String>,
) -> bool {
    facts.unknown_callers.iter().any(|caller| match caller {
        None => true,
        Some(caller) => {
            reachable.contains(caller)
                || facts
                    .exported_functions
                    .iter()
                    .any(|function| function == caller)
        }
    })
}

fn exported_symbol_scope(facts: &crate::codebase::ts_source::facts::TsFileFacts, scope: &str) -> bool {
    facts.symbols.as_ref().is_some_and(|symbols| {
        symbols
            .exports
            .iter()
            .any(|export| export.local.as_deref().unwrap_or(export.name.as_str()) == scope)
    })
}

fn reachable_function_scopes(
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
) -> HashSet<String> {
    let known_scopes = known_function_scopes(facts);
    let mut by_caller: HashMap<Option<String>, Vec<String>> = HashMap::new();
    for call in &facts.function_calls {
        by_caller
            .entry(call.caller.clone())
            .or_default()
            .push(resolve_callee_scope(
                call.caller.as_deref(),
                &call.callee,
                &known_scopes,
            ));
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

fn known_function_scopes(facts: &crate::codebase::ts_source::facts::TsFileFacts) -> HashSet<String> {
    let mut scopes: HashSet<String> = facts
        .imports
        .iter()
        .filter_map(|import| import.function_scope.clone())
        .collect();
    scopes.extend(facts.exported_functions.iter().cloned());
    scopes.extend(facts.function_calls.iter().filter_map(|call| call.caller.clone()));
    scopes
}

fn resolve_callee_scope(caller: Option<&str>, callee: &str, known_scopes: &HashSet<String>) -> String {
    if let Some(caller) = caller {
        let nested = format!("{caller}/{callee}");
        if known_scopes.contains(&nested) {
            return nested;
        }
        let mut parent = caller;
        while let Some((scope, _)) = parent.rsplit_once('/') {
            let sibling = format!("{scope}/{callee}");
            if known_scopes.contains(&sibling) {
                return sibling;
            }
            parent = scope;
        }
    }
    callee.to_string()
}

fn bare_module_node(specifier: &str) -> Option<NodeId> {
    if specifier.starts_with('.')
        || specifier.starts_with('/')
        || specifier.starts_with('#')
        || specifier.starts_with("node:")
    {
        return None;
    }
    Some(NodeId::Module(specifier.to_string()))
}
