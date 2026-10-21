//! Array liveness uses authored offsets, not call execution order. A write in a
//! hoisted function declared after its consumer is still outside this approximation.
use super::*;

impl ImportCollector {
    pub(in super::super::super) fn note_array_value(
        &mut self,
        scope: usize,
        name: &str,
        at: u32,
        value: &ValueExpression,
    ) {
        if let ValueExpression::Array(items) = value {
            let guarded = self.receiver_write_is_conditional
                || self.conditional_execution
                || self.repeat_depth > self.call_repeat_base;
            self.receiver_escapes.guarded_arrays |= guarded;
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
    let (declared_at, items) = live_items(collector, value, before)?;
    let mut found = Vec::new();
    for (key, element) in items.iter() {
        let slot = key.parse::<usize>().ok()?;
        flatten(element, slot, &mut found);
    }
    if let ValueExpression::Named { name, scope, .. } = value {
        append_pushes(
            collector,
            scope.unwrap_or(0),
            name,
            declared_at,
            before,
            &mut found,
        );
    }
    found.sort_by_key(|(slot, _)| *slot);
    Some(found)
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

fn append_pushes(
    collector: &ImportCollector,
    scope: usize,
    name: &str,
    declared_at: u32,
    before: u32,
    found: &mut Vec<(usize, ValueExpression)>,
) {
    let mut next = found.iter().map(|(slot, _)| slot + 1).max().unwrap_or(0);
    for index in collector
        .receiver_escapes
        .pushes
        .get(&scope)
        .and_then(|names| names.get(name))
        .into_iter()
        .flatten()
    {
        #[cfg(feature = "test-instrumentation")]
        alternatives::record(1, 1);
        let call = &collector.value_flow.calls[*index];
        if call.offset <= declared_at || call.offset >= before {
            continue;
        }
        for argument in call.arguments.iter() {
            #[cfg(feature = "test-instrumentation")]
            alternatives::record(2, 1);
            flatten(argument, next, found);
            next += 1;
        }
    }
}

impl ImportCollector {
    pub(in super::super::super) fn note_array_push(&mut self, call: &ValueCall) {
        let ValueExpression::Member(receiver, method) = &call.callee else {
            return;
        };
        if method != "push" {
            return;
        }
        let ValueExpression::Named { name, scope, .. } = receiver.as_ref() else {
            return;
        };
        self.receiver_escapes
            .pushes
            .entry(scope.unwrap_or(0))
            .or_default()
            .entry(name.clone())
            .or_default()
            .push(self.value_flow.calls.len());
    }
}

fn flatten(value: &ValueExpression, slot: usize, found: &mut Vec<(usize, ValueExpression)>) {
    match value {
        ValueExpression::Either(left, right) => {
            flatten(left, slot, found);
            flatten(right, slot, found);
        }
        other => found.push((slot, other.clone())),
    }
}

#[path = "escaped_array_alternatives.rs"]
mod alternatives;
pub(super) use alternatives::{Recipient, ReplayIndex};
