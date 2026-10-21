use super::*;

/// Only origins absent from the historical latest-array projection use the
/// possible channel. Unconditional writes still discard all preceding arrays.
pub(in super::super) fn additional_entries(
    collector: &ImportCollector,
    value: &ValueExpression,
    before: u32,
) -> Option<(Vec<(usize, ValueExpression)>, usize)> {
    let ValueExpression::Named { name, scope, .. } = value else {
        return None;
    };
    let scope = scope.unwrap_or(0);
    let mut events = std::collections::BTreeMap::new();
    for index in collector
        .value_binding_indexes
        .get(&scope)
        .and_then(|names| names.get(name))
        .into_iter()
        .flatten()
    {
        let binding = &collector.value_flow.bindings[*index];
        if let ValueExpression::Array(items) = &binding.value {
            if binding.declared_at <= before {
                events.insert(binding.declared_at, (items, false));
            }
        }
    }
    for (entry_scope, entry_name, at, items, guarded) in &collector.receiver_escapes.arrays {
        if *entry_scope == scope && entry_name == name && *at <= before {
            events.insert(*at, (items, *guarded));
        }
    }
    let latest = *events.last_key_value()?.0;
    let reset = events
        .iter()
        .rev()
        .find(|(_, (_, guarded))| !guarded)
        .map_or(0, |(at, _)| *at);
    let baseline = entries(collector, value, before)?;
    let baseline_length = baseline.iter().map(|(slot, _)| slot + 1).max().unwrap_or(0);
    let mut minimum_length = baseline.iter().map(|(slot, _)| slot + 1).max().unwrap_or(0);
    let mut additional = Vec::new();
    for (at, (items, _)) in events.range(reset..latest) {
        let mut candidate = Vec::new();
        for (key, element) in items.iter() {
            flatten(element, key.parse().ok()?, &mut candidate);
        }
        append_pushes(collector, scope, name, *at, before, &mut candidate);
        minimum_length = minimum_length.min(
            candidate
                .iter()
                .map(|(slot, _)| slot + 1)
                .max()
                .unwrap_or(0),
        );
        additional.extend(
            candidate
                .into_iter()
                .filter(|item| !baseline.contains(item)),
        );
    }
    (!additional.is_empty() || minimum_length != baseline_length)
        .then_some((additional, minimum_length))
}
