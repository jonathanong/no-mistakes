use super::*;

pub(super) fn forwarded_alias(collector: &ImportCollector, key: &(usize, String)) -> bool {
    // Ordinary reference traversal already forwards this alias with its saved
    // acquisition coordinates. Retaining it again would duplicate root work.
    collector
        .value_binding_indexes
        .get(&key.0)
        .and_then(|names| names.get(&key.1))
        .is_some_and(|indices| {
            indices.iter().any(|index| {
                matches!(
                    &collector.value_flow.bindings[*index].value,
                    ValueExpression::Named { name, .. } if !name.contains('.')
                )
            })
        })
}

/// Direct-call arguments are already prepared facts. This projection is used
/// only for a parameter stored into a foreign receiver member, not for generic
/// writes through arbitrary parameters.
pub(super) struct MemberArguments {
    values: FxHashMap<(usize, String), Vec<ValueExpression>>,
}

impl MemberArguments {
    pub(super) fn new(collector: &ImportCollector) -> Self {
        let mut parameters: FxHashMap<CallableId, FxHashMap<usize, (usize, String)>> = fx_map();
        for parameter in &collector.value_flow.parameters {
            // Destructuring/default parameter effects retain their existing
            // resolver semantics; this summary handles direct RHS parameters.
            if parameter.projection.is_empty() {
                parameters
                    .entry(parameter.owner)
                    .or_default()
                    .insert(parameter.index, (parameter.scope, parameter.name.clone()));
            }
        }
        let mut values: FxHashMap<_, Vec<_>> = fx_map();
        for call in &collector.value_flow.calls {
            let owner = match &call.callee {
                ValueExpression::Callable(id) => Some(*id),
                ValueExpression::Named { name, scope, .. } => {
                    collector.callable_binding_at(scope.unwrap_or(0), name)
                }
                _ => None,
            };
            let Some(slots) = owner.and_then(|owner| parameters.get(&owner)) else {
                continue;
            };
            // Walk present arguments once; absent slots must not multiply work
            // by the callable's arity for every invocation.
            for (index, argument) in call.arguments.iter().enumerate() {
                if let Some(key) = slots.get(&index) {
                    values
                        .entry(key.clone())
                        .or_default()
                        .push(argument.clone());
                }
            }
        }
        Self { values }
    }

    pub(super) fn expand(
        &self,
        references: &mut Vec<(nested::ReceiverWrite, Option<CallableId>, (u32, u32))>,
    ) {
        let mut arguments = Vec::new();
        for ((receiver, reads, _), owner, interval) in references.iter() {
            let Some(values) = self.values.get(&receiver.0) else {
                continue;
            };
            for value in values {
                let mut projected = Vec::new();
                nested::write_receiver_references(value, &receiver.1, reads, &mut projected);
                arguments.extend(
                    projected
                        .into_iter()
                        .map(|reference| (reference, *owner, *interval)),
                );
            }
        }
        references.extend(arguments);
    }
}
