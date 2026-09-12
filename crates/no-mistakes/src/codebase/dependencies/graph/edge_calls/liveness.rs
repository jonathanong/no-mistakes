fn invocation_offsets_from_bindings(
    bindings: &FxHashMap<(usize, String), crate::codebase::dependencies::extract::CallableId>,
    calls: &[crate::codebase::dependencies::extract::FunctionCall],
) -> FxHashMap<crate::codebase::dependencies::extract::CallableId, Vec<u32>> {
    let mut offsets: FxHashMap<crate::codebase::dependencies::extract::CallableId, Vec<u32>> =
        fx_map();
    for call in calls {
        if call.is_callback
            && call.invocation
                != crate::codebase::dependencies::extract::InvocationKind::Callback
        {
            continue;
        }
        let Some(scope) = call.callee_binding_scope else {
            continue;
        };
        let binding = call
            .callee
            .split_once('.')
            .map_or(call.callee.as_str(), |(binding, _)| binding);
        if let Some(id) = bindings.get(&(scope, binding.to_string())) {
            offsets.entry(*id).or_default().push(call.offset);
        }
    }
    offsets
}

fn lexical_scope_is_nested_in(
    parents: &FxHashMap<usize, Option<usize>>,
    child: Option<usize>,
    ancestor: usize,
) -> bool {
    let mut scope = child.and_then(|scope| parents.get(&scope).copied().flatten());
    while let Some(current) = scope {
        if current == ancestor {
            return true;
        }
        scope = parents.get(&current).copied().flatten();
    }
    false
}

struct BindingLivenessQuery<'a> {
    declared_at: u32,
    invalidated_at: Option<u32>,
    call_offset: u32,
    binding_scope: usize,
    call_binding_scope: Option<usize>,
    caller_id: Option<crate::codebase::dependencies::extract::CallableId>,
    invocation_offsets:
        &'a FxHashMap<crate::codebase::dependencies::extract::CallableId, Vec<u32>>,
    lexical_parents: &'a FxHashMap<usize, Option<usize>>,
}

fn binding_live_at(query: BindingLivenessQuery<'_>) -> bool {
    if query
        .invalidated_at
        .is_some_and(|cutoff| query.call_offset >= cutoff)
    {
        return false;
    }
    if query.call_offset >= query.declared_at {
        return true;
    }
    if query.call_binding_scope == Some(query.binding_scope) {
        return false;
    }
    if !lexical_scope_is_nested_in(
        query.lexical_parents,
        query.call_binding_scope,
        query.binding_scope,
    ) {
        return false;
    }
    let Some(caller_id) = query.caller_id else {
        return false;
    };
    query
        .invocation_offsets
        .get(&caller_id)
        .is_none_or(|offsets| offsets.iter().all(|offset| *offset >= query.declared_at))
}
