use super::*;

/// Call-site argument identities for parameter bindings. Constructed on demand
/// from the request's collected facts and shared by receiver-write projections.
pub(in crate::codebase::dependencies) struct ParameterCallOrigins {
    values: FxHashMap<(usize, String), Vec<ValueExpression>>,
}

impl ParameterCallOrigins {
    pub(super) fn new(collector: &ImportCollector) -> Self {
        let mut slots: FxHashMap<CallableId, FxHashMap<usize, Vec<&ValueParameter>>> = fx_map();
        for parameter in &collector.value_flow.parameters {
            slots
                .entry(parameter.owner)
                .or_default()
                .entry(parameter.index)
                .or_default()
                .push(parameter);
        }
        let mut values: FxHashMap<(usize, String), Vec<ValueExpression>> = fx_map();
        for call in &collector.value_flow.calls {
            if let ValueExpression::Member(receiver, method) = &call.callee {
                if method == "forEach" {
                    if let (ValueExpression::Array(items), Some(callback)) =
                        (receiver.as_ref(), call.arguments.first())
                    {
                        if let Some(parameters) =
                            callable_owner(collector, callback).and_then(|owner| slots.get(&owner))
                        {
                            if let Some(parameters) = parameters.get(&0) {
                                let mut entries: Vec<_> = items.iter().collect();
                                entries.sort_by(|left, right| left.0.cmp(right.0));
                                for (_, value) in entries {
                                    for parameter in parameters {
                                        append_origin(&mut values, parameter, value.clone());
                                    }
                                }
                            }
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
                if let Some(parameters) = parameters.get(&index) {
                    for parameter in parameters {
                        append_origin(&mut values, parameter, argument.clone());
                    }
                }
            }
        }
        for parameter in &collector.value_flow.parameters {
            if !matches!(parameter.default, ValueExpression::Unknown) {
                append_origin(&mut values, parameter, parameter.default.clone());
            }
        }
        Self { values }
    }

    pub(super) fn values(&self, key: &(usize, String)) -> Option<&[ValueExpression]> {
        self.values.get(key).map(Vec::as_slice)
    }
}

fn append_origin(
    values: &mut FxHashMap<(usize, String), Vec<ValueExpression>>,
    parameter: &ValueParameter,
    mut value: ValueExpression,
) {
    for projection in &parameter.projection {
        value = match projection {
            ValueProjection::Member(member) | ValueProjection::Element(member) => {
                ValueExpression::Member(std::sync::Arc::new(value), member.clone())
            }
            ValueProjection::Default(fallback, _) => ValueExpression::Default(
                std::sync::Arc::new(value),
                std::sync::Arc::new(fallback.clone()),
            ),
        };
    }
    values
        .entry((parameter.scope, parameter.name.clone()))
        .or_default()
        .push(value);
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
