#[derive(Clone)]
struct ReachabilityTransition {
    callee: crate::codebase::dependencies::extract::CallableId,
    requires_constructed_caller: bool,
    constructs_callee: bool,
}

fn reachable_function_scopes(
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
) -> HashSet<crate::codebase::dependencies::extract::CallableId> {
    let known_scopes = known_function_scopes(facts);
    let index = CallableFileIndex::from_facts(facts);
    let mut by_caller: HashMap<Option<crate::codebase::dependencies::extract::CallableId>, Vec<ReachabilityTransition>> = HashMap::new();
    for call in facts.function_calls.iter().filter(|call| {
        // Synthetic callbacks are ownership facts, not module execution
        // evidence. Preserve the historical conservative edge from an
        // executing parent into its nested anonymous callback, but do not make
        // a merely declared module callback execution-reachable. Aggregate
        // member callbacks remain excluded, except a constructor after its
        // parent was constructed. A derived class's synthetic
        // base-construction transition is likewise reachable only after its
        // derived class was constructed.
        !call.is_callback
            || (call.invocation
                == crate::codebase::dependencies::extract::InvocationKind::Membership
                && call.callee == "constructor")
            || call.invocation
                == crate::codebase::dependencies::extract::InvocationKind::Construct
            || (call.invocation
                == crate::codebase::dependencies::extract::InvocationKind::Callback
                && (call.caller.is_some() || !call.callee.starts_with("<anonymous:")))
    }) {
        let Some(callee) = reachable_callee_scope(facts, call, &known_scopes, &index) else {
            continue;
        };
        by_caller
            .entry(call.caller_id)
            .or_default()
            .push(ReachabilityTransition {
                callee,
                requires_constructed_caller: call.invocation
                    == crate::codebase::dependencies::extract::InvocationKind::Membership
                    || (call.is_callback
                        && call.invocation
                            == crate::codebase::dependencies::extract::InvocationKind::Construct),
                constructs_callee: call.invocation
                    == crate::codebase::dependencies::extract::InvocationKind::Construct,
            });
    }

    let mut reachable = HashSet::new();
    let mut visited = HashSet::new();
    let mut queue: VecDeque<(crate::codebase::dependencies::extract::CallableId, bool)> = by_caller
        .get(&None)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|transition| !transition.requires_constructed_caller)
        .map(|transition| (transition.callee, transition.constructs_callee))
        .collect();
    while let Some((function, constructed)) = queue.pop_front() {
        if !visited.insert((function, constructed)) {
            continue;
        }
        reachable.insert(function);
        if let Some(callees) = by_caller.get(&Some(function)) {
            for transition in callees {
                if !transition.requires_constructed_caller || constructed {
                    queue.push_back((transition.callee, transition.constructs_callee));
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
    index: &CallableFileIndex,
) -> Option<crate::codebase::dependencies::extract::CallableId> {
    use crate::codebase::dependencies::extract::CallTargetIdentity;

    // Canonical immutable aliases win over the raw syntactic classification:
    // a declaration prepass can prove the alias is locally bound before its
    // target has been visited, but only resolution proves which function runs.
    if let Some(resolved) = index.resolve_alias(
        call.caller.as_deref(),
        call.callee_binding_scope,
        &call.callee,
        call.offset,
        call.caller_id,
        call.invocation,
    ) {
        if let Some(id) = resolved.callable_id {
            return Some(id);
        }
        let scope = resolve_callee_scope(call.caller.as_deref(), &resolved.callee, known_scopes);
        if known_scopes.contains(&scope) {
            return callable_id_for_scope(facts, index, &scope, call.callee_binding_scope);
        }
    }

    if let Some(id) = index
        .resolve_this_member(
            call.caller.as_deref(),
            call.caller_id,
            &call.callee,
            call.invocation,
        )
        .and_then(|resolved| resolved.callable_id)
    {
        return Some(id);
    }

    if call.target_identity == CallTargetIdentity::RepositoryFunction {
        if let Some(id) = index
            .resolve_class_binding(
                call.callee_binding_scope,
                &call.callee,
                call.invocation,
            )
            .and_then(|resolved| resolved.callable_id)
        {
            return Some(id);
        }
        let scope = resolve_callee_scope(
            call.caller.as_deref(),
            &call.callee,
            known_scopes,
        );
        return callable_id_for_scope(facts, index, &scope, call.callee_binding_scope);
    }

    None
}

fn callable_id_for_scope(
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
    index: &CallableFileIndex,
    scope: &str,
    binding_scope: Option<usize>,
) -> Option<crate::codebase::dependencies::extract::CallableId> {
    if let Some(id) = binding_scope.and_then(|scope_id| {
        let name = scope.rsplit('/').next().unwrap_or(scope);
        index
            .callable_bindings
            .get(&(scope_id, name.to_string()))
            .copied()
    }) {
        return Some(id);
    }
    // Display scopes are intentionally non-unique. The caller identity drives
    // traversal; an unqualified target must be unambiguous before it can add a
    // reachability edge, rather than accidentally joining sibling declarations.
    let mut ids = facts
        .callable_scope_ids
        .iter()
        .filter_map(|(id, display)| (display == scope).then_some(*id));
    let first = ids.next()?;
    ids.next().is_none().then_some(first)
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
        let nested_member = format!("{caller}/{dotted}");
        if known_scopes.contains(&nested_member) {
            return nested_member;
        }
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
