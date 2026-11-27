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
    let mut assignments: FxHashMap<Key, (Option<CallableId>, bool, Vec<_>)> = fx_map();
    for event in &collector.receiver_escapes.reassigned {
        let (context, mixed, events) = assignments
            .entry(event.key.clone())
            .or_insert_with(|| (event.context, false, Vec::new()));
        *mixed |= *context != event.context;
        events.push(event);
    }
    if calls.is_empty() && assignments.is_empty() {
        return possible;
    }
    for (_, _, events) in assignments.values_mut() {
        events.sort_by_key(|event| event.completed_at);
    }
    for call in &collector.value_flow.calls {
        let guarded = calls.contains(&(call.offset, call.end));
        // No history keeps the existing projection. A history only proves one
        // callee when its acquisition is certain and in this execution owner.
        let acquired = key(&call.callee).and_then(|key| assignments.get(&key)).map(
            |(context, mixed, events)| {
                if *mixed || *context != call.context {
                    return None;
                }
                let before = events.partition_point(|event| event.completed_at <= call.offset);
                events[..before]
                    .last()
                    .filter(|event| event.certain)
                    .and_then(|event| super::super::callable_owner(collector, &event.value))
            },
        );
        if !guarded && acquired.is_none() {
            continue;
        }
        for owner in owners(collector, values, &call.callee) {
            if !guarded && acquired == Some(Some(owner)) {
                continue;
            }
            if let Some(parameters) = slots.get(&owner) {
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

// Match the forwarding pass's callable origins, without treating a reference
// to an assigned callback as proof that its invocation ran.
fn owners(
    collector: &ImportCollector,
    values: &FxHashMap<Key, Vec<ValueExpression>>,
    callee: &ValueExpression,
) -> Vec<CallableId> {
    super::super::callable_owner(collector, callee)
        .into_iter()
        .chain(
            key(callee)
                .and_then(|key| values.get(&key))
                .into_iter()
                .flatten()
                .filter_map(|origin| super::super::callable_owner(collector, origin)),
        )
        .collect()
}

/// Source coordinates identify named acquisitions without cloning argument trees
/// or comparing every pair of origins. Each call argument is visited once.
pub(super) fn initial(collector: &ImportCollector) -> FxHashSet<u32> {
    let mut possible = fx_set();
    if collector.receiver_escapes.conditional_calls.is_empty() {
        return possible;
    }
    for call in &collector.value_flow.calls {
        if collector
            .receiver_escapes
            .conditional_calls
            .contains(&(call.offset, call.end))
        {
            possible.extend(call.arguments.iter().filter_map(offset));
        }
    }
    possible
}

pub(super) fn offset(mut value: &ValueExpression) -> Option<u32> {
    while let ValueExpression::Member(receiver, _) = value {
        value = receiver;
    }
    match value {
        ValueExpression::Named { offset, .. } => Some(*offset),
        _ => None,
    }
}
