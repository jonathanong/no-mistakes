#[inline(never)]
fn edge_kind_for_import(import: &ExtractedImport) -> EdgeKind {
    match import.kind {
        ImportKind::Static => EdgeKind::Import,
        ImportKind::Type => EdgeKind::TypeImport,
        ImportKind::Dynamic => EdgeKind::DynamicImport,
        ImportKind::Require => EdgeKind::Require,
        ImportKind::RequireResolve => EdgeKind::RequireResolve,
    }
}

#[inline(never)]
fn import_is_reachable(
    import: &ExtractedImport,
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
    reachable: &HashSet<crate::codebase::dependencies::extract::CallableId>,
) -> bool {
    // A runtime `import()`/`require()` collected from inside an exported binding
    // initializer (e.g. `next/dynamic(() => import('./Foo'))`) lives in an
    // anonymous callback scope that no static call reaches, but it is still
    // loaded whenever the exported binding is used. The extractor flags these at
    // collection time, so treat them as reachable here.
    if import.runtime_reachable {
        return true;
    }
    let Some(scope) = import.function_scope_id else {
        return true;
    };
    facts.has_unknown_top_level_call
        || has_reachable_unknown_call(facts, reachable)
        || reachable.contains(&scope)
        || exported_function_scope(facts, import.function_scope.as_deref(), Some(scope))
        || (import.kind == ImportKind::Type
            && exported_symbol_scope(facts, import.function_scope.as_deref()))
}

#[inline(never)]
fn resource_is_reachable(
    call: &crate::codebase::ts_resources::ResourceCall,
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
    reachable: &HashSet<crate::codebase::dependencies::extract::CallableId>,
) -> bool {
    let Some(scope) = call.function_scope_id else {
        return true;
    };
    facts.has_unknown_top_level_call
        || has_reachable_unknown_call(facts, reachable)
        || reachable.contains(&scope)
        || exported_function_scope(facts, call.function_scope.as_deref(), Some(scope))
        || exported_resource_symbol_scope(facts, call.function_scope.as_deref())
}

#[inline(never)]
fn resource_diagnostic_is_reachable(
    diagnostic: &crate::codebase::ts_resources::ResourceDiagnostic,
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
    reachable: &HashSet<crate::codebase::dependencies::extract::CallableId>,
) -> bool {
    let Some(scope) = diagnostic.function_scope_id else {
        return true;
    };
    facts.has_unknown_top_level_call
        || has_reachable_unknown_call(facts, reachable)
        || reachable.contains(&scope)
        || exported_function_scope(facts, diagnostic.function_scope.as_deref(), Some(scope))
        || exported_resource_symbol_scope(facts, diagnostic.function_scope.as_deref())
}

#[inline(never)]
fn has_reachable_unknown_call(
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
    reachable: &HashSet<crate::codebase::dependencies::extract::CallableId>,
) -> bool {
    facts.unknown_calls.iter().any(|call| match call.caller_id {
        None if call.caller.is_none() => true,
        Some(id) => {
            reachable.contains(&id) || exported_function_scope(facts, call.caller.as_deref(), Some(id))
        }
        None => exported_function_scope(facts, call.caller.as_deref(), None),
    })
}

#[inline(never)]
fn exported_function_scope(
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
    scope: Option<&str>,
    scope_id: Option<crate::codebase::dependencies::extract::CallableId>,
) -> bool {
    let name_exported = facts
        .exported_functions
        .iter()
        .any(|exported| Some(exported.as_str()) == scope)
        || facts.exported_bindings.iter().any(|binding| {
            binding.specifier.is_none() && Some(binding.local.as_str()) == scope
        });
    if !name_exported {
        return false;
    }
    let Some(id) = scope_id else {
        return true;
    };
    let duplicates = facts
        .callable_scope_ids
        .iter()
        .filter(|(_, display)| Some(display.as_str()) == scope)
        .count();
    if duplicates <= 1 {
        return true;
    }
    facts
        .callable_bindings
        .iter()
        .any(|(binding_scope, name, binding_id)| {
            *binding_id == id && *binding_scope == 0 && Some(name.as_str()) == scope
        })
}

#[inline(never)]
fn exported_symbol_scope(
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
    scope: Option<&str>,
) -> bool {
    facts.symbols.as_ref().is_some_and(|symbols| {
        symbols
            .exports
            .iter()
            .any(|export| Some(export.local.as_deref().unwrap_or(export.name.as_str())) == scope)
    })
}

#[inline(never)]
fn exported_resource_symbol_scope(
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
    scope: Option<&str>,
) -> bool {
    let Some(scope) = scope else {
        return false;
    };
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
