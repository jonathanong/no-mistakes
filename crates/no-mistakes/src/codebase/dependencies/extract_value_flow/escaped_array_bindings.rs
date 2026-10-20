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
            self.receiver_escapes.arrays.push((
                scope,
                name.to_string(),
                at,
                std::sync::Arc::clone(items),
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
    for (entry_scope, entry_name, at, items) in &collector.receiver_escapes.arrays {
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
