fn resolve_callable_alias(
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
    call: &FunctionCall,
    invocation_offsets: &HashMap<crate::codebase::dependencies::extract::CallableId, Vec<u32>>,
) -> Option<String> {
    let callee = &call.callee;
    if callee.contains('.') {
        return None;
    }
    let mut binding_scope = call.callee_binding_scope?;
    let parents: HashMap<_, _> = facts.lexical_scope_parents.iter().copied().collect();
    let declared_at: HashMap<_, _> = facts
        .callable_binding_declared_at
        .iter()
        .map(|(scope, name, offset)| ((*scope, name.clone()), *offset))
        .collect();
    let mut target = callee.to_string();
    let mut resolved_alias = false;
    let mut visited = HashSet::new();
    loop {
        let mut scope = Some(binding_scope);
        let alias = loop {
            let Some(candidate_scope) = scope else { break None };
            if let Some(alias) = facts.callable_aliases.iter().find(|alias| {
                alias.binding_scope == candidate_scope
                    && alias.local == target
                    && binding_live_at(BindingLivenessQuery {
                        declared_at: alias.declared_at,
                        invalidated_at: alias.invalidated_at,
                        call_offset: call.offset,
                        binding_scope: alias.binding_scope,
                        call_binding_scope: call.callee_binding_scope,
                        caller_id: call.caller_id,
                        invocation_offsets,
                        lexical_parents: &parents,
                    })
            }) {
                break Some(alias);
            }
            if !resolved_alias {
                break None;
            }
            scope = parents.get(&candidate_scope).copied().flatten();
        };
        let Some(alias) = alias else {
            return resolved_alias
                .then(|| {
                    fact_target_binding_live(
                        facts,
                        &declared_at,
                        &parents,
                        invocation_offsets,
                        binding_scope,
                        &target,
                        call,
                    )
                    .then_some(target)
                })
                .flatten();
        };
        let key = (alias.binding_scope, alias.local.clone());
        if !visited.insert(key) {
            return None;
        }
        resolved_alias = true;
        binding_scope = alias.binding_scope;
        target = alias.target.clone();
    }
}

fn fact_target_binding_live(
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
    declared_at: &HashMap<(usize, String), u32>,
    parents: &HashMap<usize, Option<usize>>,
    invocation_offsets: &HashMap<crate::codebase::dependencies::extract::CallableId, Vec<u32>>,
    mut scope: usize,
    name: &str,
    call: &FunctionCall,
) -> bool {
    loop {
        if let Some(offset) = declared_at.get(&(scope, name.to_string())) {
            return binding_live_at(BindingLivenessQuery {
                declared_at: *offset,
                invalidated_at: None,
                call_offset: call.offset,
                binding_scope: scope,
                call_binding_scope: call.callee_binding_scope,
                caller_id: call.caller_id,
                invocation_offsets,
                lexical_parents: parents,
            });
        }
        if facts
            .callable_bindings
            .iter()
            .any(|(candidate, binding, _)| *candidate == scope && binding == name)
        {
            return true;
        }
        match parents.get(&scope).copied().flatten() {
            Some(parent) => scope = parent,
            None => return true,
        }
    }
}
