use super::*;

/// The pre-optimization replay is an independent oracle for saved small cases.
pub(super) fn projection(program: &Program<'_>, source: &str) -> Vec<(usize, ValueExpression)> {
    let mut collector = ImportCollector {
        collect_value_flow: true,
        line_starts: import_line_starts(source),
        ..Default::default()
    };
    collector.local_stack.push(fx_set());
    collector.type_local_stack.push(fx_set());
    collector.lexical_scope_ids.push(0);
    collector.lexical_scope_parents.insert(0, None);
    collector.next_lexical_scope_id = 1;
    predeclare_program_value_bindings(&mut collector, program);
    collector.visit_program(program);
    let before = collector
        .value_flow
        .calls
        .iter()
        .find(|call| call.source_callee == "mutate")
        .unwrap()
        .offset;
    let value = ValueExpression::Named {
        name: "args".into(),
        scope: Some(0),
        offset: before,
        context: None,
    };
    replay(&collector, &value, before)
}

fn replay(
    collector: &ImportCollector,
    value: &ValueExpression,
    before: u32,
) -> Vec<(usize, ValueExpression)> {
    let ValueExpression::Named { name, scope, .. } = value else {
        unreachable!()
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
    let latest = *events.last_key_value().unwrap().0;
    let reset = events
        .iter()
        .rev()
        .find(|(_, (_, guarded))| !guarded)
        .map_or(0, |(at, _)| *at);
    let baseline = entries(collector, value, before).unwrap();
    let mut additional = Vec::new();
    for (at, (items, _)) in events.range(reset..latest) {
        let mut candidate = Vec::new();
        for (key, element) in items.iter() {
            flatten(element, key.parse().unwrap(), &mut candidate);
        }
        // Deliberately retain the original full call scan and per-event replay.
        let mut next = candidate
            .iter()
            .map(|(slot, _)| slot + 1)
            .max()
            .unwrap_or(0);
        for call in &collector.value_flow.calls {
            if call.offset <= *at || call.offset >= before {
                continue;
            }
            let ValueExpression::Member(receiver, method) = &call.callee else {
                continue;
            };
            let ValueExpression::Named {
                name: receiver,
                scope: owner,
                ..
            } = receiver.as_ref()
            else {
                continue;
            };
            if method != "push" || receiver != name || owner.unwrap_or(0) != scope {
                continue;
            }
            for argument in call.arguments.iter() {
                flatten(argument, next, &mut candidate);
                next += 1;
            }
        }
        additional.extend(
            candidate
                .into_iter()
                .filter(|item| !baseline.contains(item)),
        );
    }
    additional
}
