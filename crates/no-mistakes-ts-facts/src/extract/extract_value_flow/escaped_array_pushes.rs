use super::*;

pub(super) fn append_pushes(
    collector: &ImportCollector,
    scope: usize,
    name: &str,
    declared_at: u32,
    before: u32,
    found: &mut Vec<(usize, ValueExpression)>,
    optional: &mut Option<usize>,
) {
    let mut next = found.iter().map(|(slot, _)| slot + 1).max().unwrap_or(0);
    for index in collector
        .receiver_escapes
        .pushes
        .get(&scope)
        .and_then(|names| names.get(name))
        .into_iter()
        .flatten()
    {
        #[cfg(feature = "test-instrumentation")]
        super::alternatives::record(1, 1);
        let call = &collector.value_flow.calls[*index];
        if call.offset <= declared_at || call.offset >= before {
            continue;
        }
        for argument in call.arguments.iter() {
            #[cfg(feature = "test-instrumentation")]
            super::alternatives::record(2, 1);
            if collector
                .receiver_escapes
                .optional_pushes
                .contains(&call.offset)
            {
                optional.get_or_insert(next);
            }
            super::flatten(argument, next, found);
            next += 1;
        }
    }
}

impl ImportCollector {
    pub(in super::super::super) fn note_array_push(&mut self, call: &ValueCall, ordinary: bool) {
        let ValueExpression::Member(receiver, method) = &call.callee else {
            return;
        };
        if method != "push" {
            return;
        }
        let ValueExpression::Named { name, scope, .. } = receiver.as_ref() else {
            return;
        };
        // Immutable arrays use value bindings rather than guarded summaries.
        let context_matches = self
            .value_binding_indexes
            .get(&scope.unwrap_or(0))
            .and_then(|names| names.get(name))
            .and_then(|indexes| {
                let before = indexes.partition_point(|index| {
                    self.value_flow.bindings[*index].declared_at < call.offset
                });
                before
                    .checked_sub(1)
                    .map(|last| &self.value_flow.bindings[indexes[last]])
            })
            .filter(|binding| matches!(binding.value, ValueExpression::Array(_)))
            .is_none_or(|binding| binding.owner == call.context);
        let certain = self
            .note_guarded_array_push(scope.unwrap_or(0), name, call, ordinary)
            .unwrap_or(
                !self.conditional_execution
                    && !self.receiver_write_is_conditional
                    && !call.repeated
                    && context_matches,
            );
        if ordinary && !certain {
            self.receiver_escapes.optional_pushes.insert(call.offset);
        }
        self.receiver_escapes
            .pushes
            .entry(scope.unwrap_or(0))
            .or_default()
            .entry(name.clone())
            .or_default()
            .push(self.value_flow.calls.len());
    }
}
