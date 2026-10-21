use super::*;

pub(in super::super::super) fn absorb_possible(
    collector: &ImportCollector,
    slots: &FxHashMap<CallableId, FxHashMap<usize, (usize, String)>>,
    values: &mut FxHashMap<(usize, String), Vec<ValueExpression>>,
) {
    let side = &collector.receiver_escapes;
    if side.possible_escaped.is_empty()
        && side.possible_open.is_empty()
        && side.guarded_escaped.is_empty()
        && side.guarded_open.is_empty()
        && !side.methods.iter().any(|(_, _, _, _, possible)| *possible)
    {
        return;
    }
    for call in &collector.value_flow.calls {
        if let Some(parameters) =
            super::super::super::parameter_call_origins::callable_owner(collector, &call.callee)
                .and_then(|owner| slots.get(&owner))
        {
            apply(values, parameters, collector, (call.offset, call.end));
        }
    }
    for (offset, end, method, arguments, possible) in &collector.receiver_escapes.methods {
        let Some(parameters) = slots.get(method) else {
            continue;
        };
        if *possible {
            for (index, argument) in arguments.iter().enumerate() {
                if let Some(key) = parameters.get(&index) {
                    values
                        .entry(key.clone())
                        .or_default()
                        .push(argument.clone());
                }
            }
            // Ordinary spreads at a newly resolved super call are possible too.
            super::apply_escaped(values, parameters, collector, *offset, *end);
        }
        apply(values, parameters, collector, (*offset, *end));
    }
}
fn apply(
    values: &mut FxHashMap<(usize, String), Vec<ValueExpression>>,
    parameters: &FxHashMap<usize, (usize, String)>,
    collector: &ImportCollector,
    call: (u32, u32),
) {
    for extras in [
        collector.receiver_escapes.possible_escaped.get(&call),
        collector.receiver_escapes.guarded_escaped.get(&call),
    ]
    .into_iter()
    .flatten()
    {
        for (index, extra) in extras {
            if let Some(key) = parameters.get(index) {
                values
                    .entry(key.clone())
                    .or_default()
                    .extend(extra.iter().cloned());
            }
        }
    }
    for (start, extra) in [
        collector.receiver_escapes.possible_open.get(&call),
        collector.receiver_escapes.guarded_open.get(&call),
    ]
    .into_iter()
    .flatten()
    {
        for (index, key) in parameters {
            if index >= start {
                values
                    .entry(key.clone())
                    .or_default()
                    .extend(extra.iter().cloned());
            }
        }
    }
}
