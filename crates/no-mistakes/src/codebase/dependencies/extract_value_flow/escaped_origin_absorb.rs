use super::*;

pub(in super::super) fn absorb(
    collector: &ImportCollector,
    slots: &FxHashMap<CallableId, FxHashMap<usize, (usize, String)>>,
    values: &mut FxHashMap<(usize, String), Vec<ValueExpression>>,
) -> super::super::parameter_call_origins::CaptureOrigins {
    for call in &collector.value_flow.calls {
        if let (ValueExpression::Member(receiver, method), Some(callback)) =
            (&call.callee, call.arguments.first())
        {
            let owner = super::super::parameter_call_origins::callable_owner(collector, callback);
            if matches!(
                method.as_str(),
                "forEach"
                    | "map"
                    | "filter"
                    | "some"
                    | "every"
                    | "find"
                    | "findIndex"
                    | "findLast"
                    | "findLastIndex"
                    | "flatMap"
            ) {
                if let Some(items) = super::array_elements(collector, receiver, call.offset) {
                    for item in items {
                        push_slot(values, slots, owner, 0, item);
                    }
                    if let Some(array) = callback_array(collector, receiver, call.offset) {
                        push_slot(values, slots, owner, 2, array);
                    }
                }
            }
            if matches!(method.as_str(), "reduce" | "reduceRight") {
                let parameters = owner.and_then(|id| slots.get(&id));
                if let Some(key) = parameters.and_then(|parameters| parameters.get(&0)) {
                    if let Some(initial) = call.arguments.get(1) {
                        values.entry(key.clone()).or_default().push(initial.clone());
                    } else if let Some(items) =
                        super::array_elements(collector, receiver, call.offset)
                            .filter(|items| items.len() >= 2)
                    {
                        // One element never invokes the callback. Two or more pass
                        // element 0 as the accumulator.
                        values.entry(key.clone()).or_default().extend(items);
                    }
                }
                if let Some(key) = parameters.and_then(|parameters| parameters.get(&1)) {
                    if let Some(items) = super::array_elements(collector, receiver, call.offset) {
                        values.entry(key.clone()).or_default().extend(items);
                    }
                }
            }
        }
        if let Some(parameters) =
            super::super::parameter_call_origins::callable_owner(collector, &call.callee)
                .and_then(|id| slots.get(&id))
        {
            apply_escaped(values, parameters, collector, call.offset, call.end);
        }
    }
    for (offset, end, method, arguments, possible) in &collector.receiver_escapes.methods {
        if *possible {
            continue;
        }
        if let Some(parameters) = slots.get(method) {
            for (index, argument) in arguments.iter().enumerate() {
                if let Some(key) = parameters.get(&index) {
                    values
                        .entry(key.clone())
                        .or_default()
                        .push(argument.clone());
                }
            }
            apply_escaped(values, parameters, collector, *offset, *end);
        }
    }
    let captures = super::super::parameter_call_origins::CaptureOrigins::new(collector, values);
    for event in &collector.receiver_escapes.reassigned {
        values
            .entry(event.key.clone())
            .or_default()
            .push(event.value.clone());
    }
    super::escaped_higher_order::forward(collector, slots, values);
    captures
}

fn callback_array(
    collector: &ImportCollector,
    receiver: &ValueExpression,
    before: u32,
) -> Option<ValueExpression> {
    if let ValueExpression::Array(_) = receiver {
        return Some(receiver.clone());
    }
    let ValueExpression::Named { .. } = receiver else {
        return None;
    };
    let entries = super::escaped_array_entries::array_entries(collector, receiver, before)?;
    let mut fields = fx_map();
    for (slot, value) in entries {
        fields.entry(slot.to_string()).or_insert(value);
    }
    Some(ValueExpression::Array(std::sync::Arc::new(fields)))
}

fn apply_escaped(
    values: &mut FxHashMap<(usize, String), Vec<ValueExpression>>,
    parameters: &FxHashMap<usize, (usize, String)>,
    collector: &ImportCollector,
    offset: u32,
    end: u32,
) {
    if let Some(extras) = collector.receiver_escapes.escaped.get(&(offset, end)) {
        for (index, extra) in extras {
            if let Some(key) = parameters.get(index) {
                values
                    .entry(key.clone())
                    .or_default()
                    .extend(extra.iter().cloned());
            }
        }
    }
    if let Some((start, extra)) = collector.receiver_escapes.open.get(&(offset, end)) {
        for (index, key) in parameters {
            if *index >= *start {
                values
                    .entry(key.clone())
                    .or_default()
                    .extend(extra.iter().cloned());
            }
        }
    }
}

fn push_slot(
    values: &mut FxHashMap<(usize, String), Vec<ValueExpression>>,
    slots: &FxHashMap<CallableId, FxHashMap<usize, (usize, String)>>,
    owner: Option<CallableId>,
    index: usize,
    item: ValueExpression,
) {
    if let Some(key) = owner
        .and_then(|id| slots.get(&id))
        .and_then(|parameters| parameters.get(&index))
    {
        values.entry(key.clone()).or_default().push(item);
    }
}

#[path = "escaped_possible_origins.rs"]
mod possible;
pub(in super::super) use possible::absorb_possible;
