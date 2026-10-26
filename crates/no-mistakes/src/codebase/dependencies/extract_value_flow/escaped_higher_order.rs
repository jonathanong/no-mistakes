use super::*;

/// `function apply(fn, value) { fn(value); } apply(mutate, box)` — the callee
/// is a parameter whose origins are the functions passed in.
pub(super) fn forward(
    collector: &ImportCollector,
    slots: &FxHashMap<CallableId, FxHashMap<usize, (usize, String)>>,
    values: &mut FxHashMap<(usize, String), Vec<ValueExpression>>,
) {
    for call in &collector.value_flow.calls {
        let owners = owners(collector, values, &call.callee);
        for owner in owners {
            let Some(parameters) = slots.get(&owner) else {
                continue;
            };
            let shifted = collector
                .receiver_escapes
                .uncertain_spreads
                .get(&(call.offset, call.end));
            for (index, argument) in call.arguments.iter().enumerate() {
                if shifted.is_some_and(|spread| spread.skips(index)) {
                    continue;
                }
                let Some(parameter) = parameters.get(&index) else {
                    continue;
                };
                values
                    .entry(parameter.clone())
                    .or_default()
                    .push(argument.clone());
            }
        }
    }
}

pub(super) fn owners(
    collector: &ImportCollector,
    values: &FxHashMap<(usize, String), Vec<ValueExpression>>,
    callee: &ValueExpression,
) -> Vec<CallableId> {
    let ValueExpression::Named { name, scope, .. } = callee else {
        return Vec::new();
    };
    values
        .get(&(scope.unwrap_or(0), name.clone()))
        .into_iter()
        .flatten()
        .filter_map(|origin| {
            super::super::parameter_call_origins::callable_owner(collector, origin)
        })
        .collect()
}
