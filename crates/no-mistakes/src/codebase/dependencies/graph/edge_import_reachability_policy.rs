fn edge_kind_for_import(import: &ExtractedImport) -> EdgeKind {
    match import.kind {
        ImportKind::Static => EdgeKind::Import,
        ImportKind::Type => EdgeKind::TypeImport,
        ImportKind::Dynamic => EdgeKind::DynamicImport,
        ImportKind::Require => EdgeKind::Require,
        ImportKind::RequireResolve => EdgeKind::RequireResolve,
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
