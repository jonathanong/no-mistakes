use super::*;

/// Call-site argument identities for bare parameters. Constructed on demand
/// from the request's collected facts and shared by receiver-write projections.
pub(in crate::codebase::dependencies) struct ParameterCallOrigins {
    values: FxHashMap<(usize, String), Vec<ValueExpression>>,
}

impl ParameterCallOrigins {
    pub(super) fn new(collector: &ImportCollector) -> Self {
        let mut slots: FxHashMap<CallableId, FxHashMap<usize, (usize, String)>> = fx_map();
        for parameter in &collector.value_flow.parameters {
            if parameter.projection.is_empty() {
                slots
                    .entry(parameter.owner)
                    .or_default()
                    .insert(parameter.index, (parameter.scope, parameter.name.clone()));
            }
        }
        let mut values: FxHashMap<(usize, String), Vec<ValueExpression>> = fx_map();
        for call in &collector.value_flow.calls {
            if let ValueExpression::Member(receiver, method) = &call.callee {
                if matches!(method.as_str(), "forEach" | "map") {
                    if let (Some(items), Some(callback)) =
                        (array_elements(collector, receiver), call.arguments.first())
                    {
                        if let Some(key) = callable_owner(collector, callback)
                            .and_then(|owner| slots.get(&owner))
                            .and_then(|parameters| parameters.get(&0))
                        {
                            values.entry(key.clone()).or_default().extend(items);
                        }
                    }
                }
                if matches!(method.as_str(), "reduce" | "reduceRight") {
                    if let (Some(callback), Some(initial)) =
                        (call.arguments.first(), call.arguments.get(1))
                    {
                        if let Some(key) = callable_owner(collector, callback)
                            .and_then(|owner| slots.get(&owner))
                            .and_then(|parameters| parameters.get(&0))
                        {
                            values.entry(key.clone()).or_default().push(initial.clone());
                        }
                    }
                }
            }
            let Some(parameters) =
                callable_owner(collector, &call.callee).and_then(|owner| slots.get(&owner))
            else {
                continue;
            };
            for (index, argument) in call.arguments.iter().enumerate() {
                if let Some(key) = parameters.get(&index) {
                    values
                        .entry(key.clone())
                        .or_default()
                        .push(argument.clone());
                }
            }
            if let Some(extras) = collector.escaped_parameter_arguments.get(&call.offset) {
                for (index, extra) in extras {
                    if let Some(key) = parameters.get(index) {
                        values
                            .entry(key.clone())
                            .or_default()
                            .extend(extra.iter().cloned());
                    }
                }
            }
        }
        for (owner, arguments) in &collector.class_method_calls {
            let Some(parameters) = slots.get(owner) else {
                continue;
            };
            for (index, argument) in arguments.iter().enumerate() {
                if let Some(key) = parameters.get(&index) {
                    values
                        .entry(key.clone())
                        .or_default()
                        .push(argument.clone());
                }
            }
        }
        for (scope, name, value) in &collector.parameter_reassignments {
            values
                .entry((*scope, name.clone()))
                .or_default()
                .push(value.clone());
        }
        for parameter in &collector.value_flow.parameters {
            if parameter.projection.is_empty()
                && !matches!(parameter.default, ValueExpression::Unknown)
            {
                values
                    .entry((parameter.scope, parameter.name.clone()))
                    .or_default()
                    .push(parameter.default.clone());
            }
        }
        Self { values }
    }

    pub(super) fn values(&self, key: &(usize, String)) -> Option<&[ValueExpression]> {
        self.values.get(key).map(Vec::as_slice)
    }
}

impl ImportCollector {
    pub(super) fn parameter_call_origins(&mut self) -> std::sync::Arc<ParameterCallOrigins> {
        if let Some(origins) = &self.parameter_call_origins {
            return std::sync::Arc::clone(origins);
        }
        let origins = std::sync::Arc::new(ParameterCallOrigins::new(self));
        self.parameter_call_origins = Some(std::sync::Arc::clone(&origins));
        origins
    }
}

pub(super) fn array_elements(
    collector: &ImportCollector,
    value: &ValueExpression,
) -> Option<Vec<ValueExpression>> {
    match value {
        ValueExpression::Array(items) => Some(sorted_array(items)),
        ValueExpression::Named { name, scope, .. } => {
            let scope = scope.unwrap_or(0);
            let indexes = collector.value_binding_indexes.get(&scope)?.get(name)?;
            // Appended with the binding, so the last index addresses that value.
            let index = indexes[indexes.len() - 1];
            match &collector.value_flow.bindings[index].value {
                ValueExpression::Array(items) => Some(sorted_array(items)),
                _ => None,
            }
        }
        _ => None,
    }
}

fn sorted_array(items: &FxHashMap<String, ValueExpression>) -> Vec<ValueExpression> {
    let mut entries: Vec<_> = items.iter().collect();
    entries.sort_by(|left, right| left.0.cmp(right.0));
    entries
        .into_iter()
        .map(|(_, value)| value.clone())
        .collect()
}

fn callable_owner(collector: &ImportCollector, callee: &ValueExpression) -> Option<CallableId> {
    match callee {
        ValueExpression::Callable(id) => Some(*id),
        ValueExpression::Named { name, scope, .. } => {
            collector.callable_binding_at(scope.unwrap_or(0), name)
        }
        ValueExpression::Member(receiver, member) => {
            let owner = callable_owner(collector, receiver)?;
            owner_member_id(&collector.aggregate_callable_member_ids, owner, member)
        }
        _ => None,
    }
}
