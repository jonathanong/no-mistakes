#[inline(never)]
fn call_target_identity(
    file: &CallableFileIndex,
    call: &crate::codebase::dependencies::extract::FunctionCall,
    callee: &str,
) -> crate::codebase::dependencies::extract::CallTargetIdentity {
    if callee == call.callee {
        return call.target_identity;
    }
    let binding = callee
        .split_once('.')
        .map_or(callee, |(binding, _)| binding);
    if file.imported.contains_key(binding) {
        crate::codebase::dependencies::extract::CallTargetIdentity::ModuleExport
    } else if resolve_local_call_scope(
        call.caller.as_deref(),
        call.callee_binding_scope,
        callee,
        &file.known_scopes,
        &file.class_scopes,
    )
    .is_some()
    {
        crate::codebase::dependencies::extract::CallTargetIdentity::RepositoryFunction
    } else {
        crate::codebase::dependencies::extract::CallTargetIdentity::Unknown
    }
}

/// Resolves an unqualified name through lexical parent scopes, matching the
/// existing import-reachability scope behavior without another AST pass.
#[inline(never)]
fn resolve_local_call_scope<'a>(
    caller: Option<&str>,
    _binding_scope: Option<usize>,
    callee: &str,
    known: &'a FxHashSet<String>,
    class_scopes: &FxHashSet<String>,
) -> Option<&'a str> {
    let binding = callee
        .split_once('.')
        .map_or(callee, |(binding, _)| binding);
    if callee.contains('.') && known.contains(binding) && !class_scopes.contains(binding) {
        return None;
    }
    let mut scope = caller;
    while let Some(current) = scope {
        let candidate = format!("{current}/{callee}");
        if known.contains(candidate.as_str()) {
            return known.get(candidate.as_str()).map(String::as_str);
        }
        let member = callee.replace('.', "/");
        let candidate = format!("{current}/{member}");
        if known.contains(candidate.as_str()) {
            return known.get(candidate.as_str()).map(String::as_str);
        }
        scope = current.rsplit_once('/').map(|(parent, _)| parent);
    }
    known.get(callee).map(String::as_str)
        .or_else(|| {
        known
            .get(callee.replace('.', "/").as_str())
            .map(String::as_str)
        })
}
