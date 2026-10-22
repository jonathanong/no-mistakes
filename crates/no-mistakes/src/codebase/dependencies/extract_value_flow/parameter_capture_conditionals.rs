use super::*;

/// Only newly forwarded capture inputs use this uncertainty projection. Keep
/// historical direct-call inputs unchanged, including their existing unions.
pub(super) fn keys(
    collector: &ImportCollector,
    slots: &FxHashMap<CallableId, FxHashMap<usize, Key>>,
    values: &FxHashMap<Key, Vec<ValueExpression>>,
) -> FxHashSet<Key> {
    let calls = &collector.receiver_escapes.conditional_calls;
    let mut possible = fx_set();
    if calls.is_empty() {
        return possible;
    }
    for call in &collector.value_flow.calls {
        if calls.contains(&(call.offset, call.end)) {
            if let Some(parameters) = super::super::callable_owner(collector, &call.callee)
                .and_then(|owner| slots.get(&owner))
            {
                possible.extend(parameters.values().cloned());
            }
        }
    }
    // Reverse parameter references once, then visit each reached key/edge once.
    // Cyclic forwarders cannot reconstruct prefixes or recurse indefinitely.
    let mut dependents: FxHashMap<Key, Vec<&Key>> = fx_map();
    for (target, inputs) in values {
        for input in inputs {
            if let Some(source) = key(input) {
                dependents.entry(source).or_default().push(target);
            }
        }
    }
    let mut pending = possible.iter().cloned().collect::<Vec<_>>();
    while let Some(source) = pending.pop() {
        for target in dependents.get(&source).into_iter().flatten() {
            if possible.insert((*target).clone()) {
                pending.push((*target).clone());
            }
        }
    }
    possible
}

/// Member arguments retain the same parameter source as their receiver. Do not
/// infer dependencies through opaque computed values or aggregate construction.
pub(super) fn key(mut value: &ValueExpression) -> Option<Key> {
    while let ValueExpression::Member(receiver, _) = value {
        value = receiver;
    }
    match value {
        ValueExpression::Named { name, scope, .. } => Some((scope.unwrap_or(0), name.clone())),
        _ => None,
    }
}
