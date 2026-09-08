#[derive(Clone)]
struct ReachabilityTransition {
    callee: String,
    requires_constructed_caller: bool,
    constructs_callee: bool,
}

fn reachable_function_scopes(
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
) -> HashSet<String> {
    let known_scopes = known_function_scopes(facts);
    let mut by_caller: HashMap<Option<String>, Vec<ReachabilityTransition>> = HashMap::new();
    for call in facts.function_calls.iter().filter(|call| {
        // Synthetic callbacks are ownership facts, not module execution
        // evidence. Preserve the historical conservative edge from an
        // executing parent into its nested anonymous callback, but do not make
        // a merely declared module callback execution-reachable. Aggregate
        // member callbacks remain excluded, except a constructor after its
        // parent was constructed.
        !call.is_callback
            || (call.invocation
                == crate::codebase::dependencies::extract::InvocationKind::Membership
                && call.callee == "constructor")
            || (call.caller.is_some() && call.callee.starts_with("<anonymous:"))
    }) {
        let Some(callee) = reachable_callee_scope(facts, call, &known_scopes) else {
            continue;
        };
        by_caller
            .entry(call.caller.clone())
            .or_default()
            .push(ReachabilityTransition {
                callee,
                requires_constructed_caller: call.invocation
                    == crate::codebase::dependencies::extract::InvocationKind::Membership,
                constructs_callee: call.invocation
                    == crate::codebase::dependencies::extract::InvocationKind::Construct,
            });
    }

    let mut reachable = HashSet::new();
    let mut visited = HashSet::new();
    let mut queue: VecDeque<(String, bool)> = by_caller
        .get(&None)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|transition| !transition.requires_constructed_caller)
        .map(|transition| (transition.callee, transition.constructs_callee))
        .collect();
    while let Some((function, constructed)) = queue.pop_front() {
        if !visited.insert((function.clone(), constructed)) {
            continue;
        }
        reachable.insert(function.clone());
        if let Some(callees) = by_caller.get(&Some(function)) {
            for transition in callees {
                if !transition.requires_constructed_caller || constructed {
                    queue.push_back((transition.callee.clone(), transition.constructs_callee));
                }
            }
        }
    }
    reachable
}

fn reachable_callee_scope(
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
    call: &FunctionCall,
    known_scopes: &HashSet<String>,
) -> Option<String> {
    use crate::codebase::dependencies::extract::CallTargetIdentity;

    // Canonical immutable aliases win over the raw syntactic classification:
    // a declaration prepass can prove the alias is locally bound before its
    // target has been visited, but only resolution proves which function runs.
    if let Some(resolved) = resolve_callable_alias(facts, call.caller.as_deref(), &call.callee) {
        let scope = resolve_callee_scope(call.caller.as_deref(), &resolved, known_scopes);
        if known_scopes.contains(&scope) {
            return Some(scope);
        }
    }

    if call.target_identity == CallTargetIdentity::RepositoryFunction {
        return Some(resolve_callee_scope(
            call.caller.as_deref(),
            &call.callee,
            known_scopes,
        ));
    }

    None
}

fn resolve_callable_alias(
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
    caller: Option<&str>,
    callee: &str,
) -> Option<String> {
    if callee.contains('.') {
        return None;
    }
    let mut owner = caller.map(str::to_string);
    let mut target = callee.to_string();
    let mut resolved_alias = false;
    let mut visited = HashSet::new();
    loop {
        let key = (owner.clone(), target.clone());
        let alias = facts
            .callable_aliases
            .iter()
            .find(|alias| alias.scope == key.0 && alias.local == key.1)
            .cloned();
        let Some(alias) = alias else {
            if !resolved_alias {
                if let Some(parent) = owner
                    .as_deref()
                    .and_then(|scope| scope.rsplit_once('/').map(|(parent, _)| parent.to_string()))
                {
                    owner = Some(parent);
                    continue;
                }
                if owner.is_some() {
                    owner = None;
                    continue;
                }
            }
            return resolved_alias.then_some(target);
        };
        if !visited.insert(key) {
            return None;
        }
        resolved_alias = true;
        target = alias.target;
    }
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

fn bare_module_node_in(interner: &PathInterner, specifier: &str) -> Option<NodeId> {
    if specifier.starts_with('.')
        || specifier.starts_with('/')
        || specifier.starts_with('#')
        || specifier.starts_with("node:")
    {
        return None;
    }
    Some(NodeId::module_in(interner, specifier))
}
