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
        || exported_function_scope(facts, scope)
        || (import.kind == ImportKind::Type && exported_symbol_scope(facts, scope))
}

fn resource_is_reachable(
    call: &crate::codebase::ts_resources::ResourceCall,
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
    reachable: &HashSet<String>,
) -> bool {
    let Some(scope) = &call.function_scope else {
        return true;
    };
    facts.has_unknown_top_level_call
        || has_reachable_unknown_call(facts, reachable)
        || reachable.contains(scope)
        || exported_function_scope(facts, scope)
        || exported_resource_symbol_scope(facts, scope)
}

fn resource_diagnostic_is_reachable(
    diagnostic: &crate::codebase::ts_resources::ResourceDiagnostic,
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
    reachable: &HashSet<String>,
) -> bool {
    let Some(scope) = &diagnostic.function_scope else {
        return true;
    };
    facts.has_unknown_top_level_call
        || has_reachable_unknown_call(facts, reachable)
        || reachable.contains(scope)
        || exported_function_scope(facts, scope)
        || exported_resource_symbol_scope(facts, scope)
}

fn has_reachable_unknown_call(
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
    reachable: &HashSet<String>,
) -> bool {
    facts.unknown_callers.iter().any(|caller| match caller {
        None => true,
        Some(caller) => reachable.contains(caller) || exported_function_scope(facts, caller),
    })
}

fn exported_function_scope(
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
    scope: &str,
) -> bool {
    facts
        .exported_functions
        .iter()
        .any(|exported| exported == scope)
}

fn exported_symbol_scope(
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
    scope: &str,
) -> bool {
    facts.symbols.as_ref().is_some_and(|symbols| {
        symbols
            .exports
            .iter()
            .any(|export| export.local.as_deref().unwrap_or(export.name.as_str()) == scope)
    })
}

fn exported_resource_symbol_scope(
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
    scope: &str,
) -> bool {
    if facts
        .exported_resource_scopes
        .iter()
        .any(|exported| exported == scope)
    {
        return true;
    }
    for exported in &facts.exported_resource_roots {
        if scope == exported {
            return true;
        }
        let Some(suffix) = scope.strip_prefix(exported) else {
            continue;
        };
        let Some(member) = suffix.strip_prefix('/') else {
            continue;
        };
        if !member.is_empty() && !member.contains('/') {
            return true;
        }
    }
    false
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

fn resolve_callee_scope(
    caller: Option<&str>,
    callee: &str,
    known_scopes: &HashSet<String>,
) -> String {
    if known_scopes.contains(callee) {
        return callee.to_string();
    }
    // The import extractor records top-level member invocations as `api.load`,
    // while nested callable scopes use slash-separated names (`api/load`).
    // Normalize only when that exact canonical scope is known.
    let dotted = callee.replace('.', "/");
    if known_scopes.contains(&dotted) {
        return dotted;
    }
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
