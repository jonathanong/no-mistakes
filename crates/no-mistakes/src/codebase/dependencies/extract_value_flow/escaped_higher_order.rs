use super::*;

/// `function apply(fn, value) { fn(value); } apply(mutate, box)` — the callee
/// is a parameter whose origins are the functions passed in.
pub(super) fn forward(
    collector: &ImportCollector,
    slots: &FxHashMap<CallableId, FxHashMap<usize, (usize, String)>>,
    values: &mut FxHashMap<(usize, String), Vec<ValueExpression>>,
) {
    for call in &collector.value_flow.calls {
        let ValueExpression::Named { name, scope, .. } = &call.callee else {
            continue;
        };
        let key = (scope.unwrap_or(0), name.clone());
        let Some(origins) = values.get(&key) else {
            continue;
        };
        let owners: Vec<CallableId> = origins
            .iter()
            .filter_map(|origin| {
                super::super::parameter_call_origins::callable_owner(collector, origin)
            })
            .collect();
        for owner in owners {
            let Some(parameters) = slots.get(&owner) else {
                continue;
            };
            for (index, argument) in call.arguments.iter().enumerate() {
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
