use super::*;

pub(super) fn append_pushes(
    collector: &ImportCollector,
    scope: usize,
    name: &str,
    declared_at: u32,
    before: u32,
    found: &mut Vec<(usize, ValueExpression)>,
    optional: &mut FxHashSet<usize>,
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
                optional.insert(next);
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
        self.note_guarded_array_push(scope.unwrap_or(0), name, call, ordinary);
        if ordinary
            && (self.conditional_execution || self.receiver_write_is_conditional || call.repeated)
        {
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
