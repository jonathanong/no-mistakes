//! Escaped array elements that can reach a later spread.
//!
//! A later unconditional assignment replaces the array. An assignment inside a
//! conditional or a loop is unioned with the array that is still live when that
//! guard does not run.
//!
//! A write inside a function whose body starts after the spread is not live for
//! that spread. The window ends at the spread's source offset, so a hoisted
//! function declared later stays outside it.

use super::*;

type ArrayItems = std::sync::Arc<FxHashMap<String, ValueExpression>>;
type LiveArray = (u32, ArrayItems, bool);

impl ImportCollector {
    pub(in super::super::super) fn note_array_value(
        &mut self,
        scope: usize,
        name: &str,
        at: u32,
        value: &ValueExpression,
    ) {
        if let ValueExpression::Array(items) = value {
            let conditional =
                self.conditional_execution || self.repeat_depth > self.call_repeat_base;
            self.receiver_escapes.arrays.push((
                scope,
                name.to_string(),
                at,
                std::sync::Arc::clone(items),
                conditional,
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
    let mut rows: Vec<LiveArray> = Vec::new();
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
                rows.push((binding.declared_at, std::sync::Arc::clone(items), false));
            }
        }
    }
    for (entry_scope, entry_name, at, items, conditional) in &collector.receiver_escapes.arrays {
        if *entry_scope != scope || entry_name != name || *at > before {
            continue;
        }
        rows.push((*at, std::sync::Arc::clone(items), *conditional));
    }
    union_live_arrays(&mut rows)
}

/// The latest unconditional array replaces everything before it. Later guarded
/// arrays stay live on the path that skips the guard.
fn union_live_arrays(rows: &mut [LiveArray]) -> Option<(u32, ArrayItems)> {
    if rows.is_empty() {
        return None;
    }
    rows.sort_by_key(|(at, _, _)| *at);
    let start = rows
        .iter()
        .rposition(|(_, _, conditional)| !conditional)
        .unwrap_or(0);
    let (declared_at, first, _) = &rows[start];
    let mut merged = (**first).clone();
    for (_, items, _) in rows.iter().skip(start + 1) {
        for (key, value) in items.iter() {
            match merged.get(key) {
                Some(existing) if existing == value => {}
                Some(existing) => {
                    let either = ValueExpression::Either(
                        std::sync::Arc::new(existing.clone()),
                        std::sync::Arc::new(value.clone()),
                    );
                    merged.insert(key.clone(), either);
                }
                None => {
                    merged.insert(key.clone(), value.clone());
                }
            }
        }
    }
    Some((*declared_at, std::sync::Arc::new(merged)))
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
    for call in &collector.value_flow.calls {
        if call.offset <= declared_at || call.offset >= before {
            continue;
        }
        let Some(arguments) = push_arguments(call, scope, name) else {
            continue;
        };
        for argument in arguments {
            flatten(argument, next, found);
            next += 1;
        }
    }
}

fn push_arguments<'a>(
    call: &'a ValueCall,
    scope: usize,
    name: &str,
) -> Option<&'a [ValueExpression]> {
    let ValueExpression::Member(receiver, method) = &call.callee else {
        return None;
    };
    if method != "push" {
        return None;
    }
    let ValueExpression::Named {
        name: receiver_name,
        scope: receiver_scope,
        ..
    } = receiver.as_ref()
    else {
        return None;
    };
    if receiver_name != name || receiver_scope.unwrap_or(0) != scope {
        return None;
    }
    Some(call.arguments.as_slice())
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
