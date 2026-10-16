use super::*;

/// A fresh per-site flow cannot refine this direct lexical function identity.
/// Keep every alias, import, projection, uncertain binding and non-singleton target.
pub(super) fn needs_refinement(
    facts: &dyn TsFactLookup,
    indexes: &CallableResolutionIndexes,
    site: &ResolvedCallSite,
    #[cfg(feature = "test-instrumentation")] probe: &admission_probe::AdmissionProbe,
) -> bool {
    #[cfg(feature = "test-instrumentation")]
    if probe.original_admission() {
        return true;
    }
    let redundant = exact_direct_callable(facts, indexes, site);
    #[cfg(feature = "test-instrumentation")]
    if redundant {
        probe.skip_direct_callable();
    }
    !redundant
}
fn exact_direct_callable(
    facts: &dyn TsFactLookup,
    indexes: &CallableResolutionIndexes,
    site: &ResolvedCallSite,
) -> bool {
    let Some((target_id, target_scope)) = exact_identity(site) else {
        return false;
    };
    let Some(file) = indexes.file(facts, &site.file) else {
        return false;
    };
    let Some(call) = file
        .values
        .sites
        .get(&site.offset)
        .and_then(|sites| sites.get(&site.source_callee))
        .and_then(|key| file.values.calls.get(key))
    else {
        return false;
    };
    exact_name(&file, call, site, target_id, target_scope)
}
fn exact_identity(site: &ResolvedCallSite) -> Option<(CallableId, &str)> {
    if site.invocation != InvocationKind::Call {
        return None;
    }
    let ResolvedCallTarget::RepositoryFunction {
        file: target_file,
        scope: target_scope,
    } = &site.target
    else {
        return None;
    };
    let Some(NodeId::Symbol {
        file: node_file,
        symbol,
        callable_id: Some(target_id),
    }) = &site.target_node
    else {
        return None;
    };
    if target_file != &site.file
        || node_file.as_ref() != site.file.as_path()
        || symbol.as_ref() != target_scope.as_str()
    {
        return None;
    }
    Some((*target_id, target_scope.as_str()))
}
fn exact_name(
    file: &CallableFileIndex,
    call: &ValueCall,
    site: &ResolvedCallSite,
    target_id: CallableId,
    target_scope: &str,
) -> bool {
    let ValueExpression::Named {
        name,
        scope: Some(scope),
        ..
    } = &call.callee
    else {
        return false;
    };
    if call.bound
        || name.contains(['.', '/'])
        || name != &site.source_callee
        || file.imported.contains_key(name)
        || scoped_name(&file.values.bindings, *scope, name).is_some()
        || scoped_name(&file.values.parameters, *scope, name).is_some()
        || scoped_name(&file.aliases, *scope, name).is_some()
        || scoped_name(&file.callable_assignments, *scope, name).is_some()
    {
        return false;
    }
    // A direct map hit is precisely the first branch of resolve_local_callable_id;
    // do not broaden this to an enclosing lexical frame or display-name match.
    scoped_name(&file.callable_bindings, *scope, name) == Some(&target_id)
        && file.values.scopes.get(&target_id).map(String::as_str) == Some(target_scope)
}

#[cfg(test)]
#[path = "known_callable_admission_tests.rs"]
mod tests;
