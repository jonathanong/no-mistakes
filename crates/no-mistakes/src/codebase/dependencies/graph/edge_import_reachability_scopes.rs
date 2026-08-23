/// Collect every canonical scope that can be a call target. Resource
/// diagnostics participate because a dynamic-only method still needs its
/// slash-separated scope (`api/load`) to resolve a dotted invocation
/// (`api.load()`) during reachability traversal.
fn known_function_scopes(
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
) -> HashSet<String> {
    let mut scopes: HashSet<String> = facts
        .imports
        .iter()
        .filter_map(|import| import.function_scope.clone())
        .collect();
    scopes.extend(
        facts
            .resource_calls
            .iter()
            .filter_map(|call| call.function_scope.clone()),
    );
    scopes.extend(
        facts
            .resource_diagnostics
            .iter()
            .filter_map(|diagnostic| diagnostic.function_scope.clone()),
    );
    scopes.extend(facts.exported_functions.iter().cloned());
    scopes.extend(
        facts
            .function_calls
            .iter()
            .filter_map(|call| call.caller.clone()),
    );
    scopes
}
