//! Array liveness uses authored offsets, not call execution order. A write in a
//! hoisted function declared after its consumer is still outside this approximation.
use super::*;
#[path = "escaped_array_pushes.rs"]
mod pushes;

impl ImportCollector {
    pub(in super::super::super) fn note_array_value(
        &mut self,
        scope: usize,
        name: &str,
        at: u32,
        ready_at: u32,
        value: &ValueExpression,
    ) {
        if let ValueExpression::Array(items) = value {
            let guarded = self.receiver_write_is_conditional
                || self.conditional_execution
                || self.repeat_depth > self.call_repeat_base;
            self.receiver_escapes.guarded_arrays |= guarded;
            self.note_guarded_array(scope, name, at, ready_at, items, guarded);
            self.receiver_escapes.arrays.push((
                scope,
                name.to_string(),
                at,
                std::sync::Arc::clone(items),
                guarded,
            ));
        }
    }
}

pub(super) fn entries(
    collector: &ImportCollector,
    value: &ValueExpression,
    before: u32,
) -> Option<Vec<(usize, ValueExpression)>> {
    Some(guarded_entries(collector, value, before)?.0)
}
pub(super) fn guarded_entries(
    collector: &ImportCollector,
    value: &ValueExpression,
    before: u32,
) -> Option<(Vec<(usize, ValueExpression)>, FxHashSet<usize>)> {
    let (declared_at, items) = live_items(collector, value, before)?;
    let mut optional = fx_set();
    let mut found = Vec::new();
    for (key, element) in items.iter() {
        let slot = key.parse::<usize>().ok()?;
        flatten(element, slot, &mut found);
    }
    if let ValueExpression::Named { name, scope, .. } = value {
        pushes::append_pushes(
            collector,
            scope.unwrap_or(0),
            name,
            declared_at,
            before,
            &mut found,
            &mut optional,
        );
    }
    found.sort_by_key(|(slot, _)| *slot);
    Some((found, optional))
}

fn live_items(
    collector: &ImportCollector,
    value: &ValueExpression,
    before: u32,
) -> Option<(u32, std::sync::Arc<FxHashMap<String, ValueExpression>>)> {
    match value {
        ValueExpression::Array(items) => Some((0, std::sync::Arc::clone(items))),
        ValueExpression::Named { name, scope, .. } => {
            named_items(collector, scope.unwrap_or(0), name, before)
        }
        _ => None,
    }
}

fn named_items(
    collector: &ImportCollector,
    scope: usize,
    name: &str,
    before: u32,
) -> Option<(u32, std::sync::Arc<FxHashMap<String, ValueExpression>>)> {
    let mut best: Option<(u32, std::sync::Arc<FxHashMap<String, ValueExpression>>)> = None;
    if let Some(indexes) = collector
        .value_binding_indexes
        .get(&scope)
        .and_then(|names| names.get(name))
    {
        for index in indexes {
            let binding = &collector.value_flow.bindings[*index];
            if binding.declared_at > before {
                continue;
            }
            if let ValueExpression::Array(items) = &binding.value {
                if best
                    .as_ref()
                    .is_none_or(|(at, _)| binding.declared_at >= *at)
                {
                    best = Some((binding.declared_at, std::sync::Arc::clone(items)));
                }
            }
        }
    }
    for (entry_scope, entry_name, at, items, _) in &collector.receiver_escapes.arrays {
        if *entry_scope != scope || entry_name != name || *at > before {
            continue;
        }
        if best.as_ref().is_none_or(|(best_at, _)| *at >= *best_at) {
            best = Some((*at, std::sync::Arc::clone(items)));
        }
    }
    best
}

fn flatten(value: &ValueExpression, slot: usize, found: &mut Vec<(usize, ValueExpression)>) {
    match value {
        ValueExpression::Either(left, right) => {
            flatten(left, slot, found);
            flatten(right, slot, found);
        }
        // SQL provenance inputs are opaque values for receiver argument replay.
        ValueExpression::Derived(_) => found.push((slot, ValueExpression::Unknown)),
        other => found.push((slot, other.clone())),
    }
}

#[path = "escaped_array_alternatives.rs"]
mod alternatives;
pub(super) use alternatives::{Recipient, ReplayIndex};
