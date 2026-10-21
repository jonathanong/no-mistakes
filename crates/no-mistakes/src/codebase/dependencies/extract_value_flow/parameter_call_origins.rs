use super::*;
#[path = "parameter_capture_origins.rs"]
mod captures;
pub(super) use captures::{CaptureOrigins, Reassignment};

/// Call-site argument identities for bare parameters. Constructed on demand
/// from the request's collected facts and shared by receiver-write projections.
pub(in crate::codebase::dependencies) struct ParameterCallOrigins {
    values: FxHashMap<(usize, String), Vec<ValueExpression>>,
    captures: captures::CaptureOrigins,
    recovered: FxHashMap<(usize, String), Vec<ValueExpression>>,
    possible: FxHashMap<(usize, String), Vec<ValueExpression>>,
}

pub(super) struct SelectedOrigins<'a> {
    pub(super) values: Option<&'a [ValueExpression]>,
    pub(super) possible: [Option<&'a [ValueExpression]>; 2],
    pub(super) uncertain: bool,
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
            let shifted = collector
                .receiver_escapes
                .uncertain_spreads
                .get(&(call.offset, call.end));
            for (index, argument) in call.arguments.iter().enumerate() {
                if shifted.is_some_and(|spread| spread.skips(index)) {
                    continue;
                }
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
        let captures = super::receiver_escapes::absorb(collector, &slots, &mut values);
        let mut possible = fx_map();
        super::receiver_escapes::absorb_possible(collector, &slots, &values, &mut possible);
        let mut recovered: FxHashMap<_, Vec<_>> = fx_map();
        for event in collector
            .receiver_escapes
            .reassigned
            .iter()
            .filter(|event| event.recovered)
        {
            recovered
                .entry(event.key.clone())
                .or_default()
                .push(event.value.clone());
        }
        Self {
            values,
            recovered,
            possible,
            captures,
        }
    }

    pub(super) fn possible_values(&self, key: &(usize, String)) -> Option<&[ValueExpression]> {
        self.possible.get(key).map(Vec::as_slice)
    }
    pub(super) fn values_at(
        &self,
        key: &(usize, String),
        read: nested::RootRead,
    ) -> SelectedOrigins<'_> {
        let selected = self.captures.at(key, read);
        SelectedOrigins {
            values: selected
                .map(|(values, _)| values)
                .or_else(|| self.values(key)),
            uncertain: selected.is_some_and(|(_, possible)| possible),
            possible: [
                self.possible_values(key),
                selected
                    .is_none()
                    .then(|| self.recovered.get(key).map(Vec::as_slice))
                    .flatten(),
            ],
        }
    }
    pub(super) fn has_values(&self, key: &(usize, String)) -> bool {
        self.values.contains_key(key)
            || self.possible.contains_key(key)
            || self.recovered.contains_key(key)
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
