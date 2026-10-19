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
        super::receiver_escapes::absorb(collector, &slots, &mut values);
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

pub(super) fn callable_owner(
    collector: &ImportCollector,
    callee: &ValueExpression,
) -> Option<CallableId> {
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
