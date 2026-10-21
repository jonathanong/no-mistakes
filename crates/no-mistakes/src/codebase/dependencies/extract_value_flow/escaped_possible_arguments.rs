use super::escaped_array_bindings::{Recipient, ReplayIndex};
use super::*;

pub(super) enum ArgumentValue {
    Spread(ValueExpression),
    Value(Vec<ValueExpression>),
}
pub(super) struct Pending {
    pub(super) span: (u32, u32),
    arguments: Vec<ArgumentValue>,
}
impl ImportCollector {
    /// Keep facts from this visit; final binding identities own their projection.
    pub(super) fn note_possible_arguments(&mut self, call: &CallExpression<'_>) {
        if !self.receiver_escapes.guarded_arrays
            || !call
                .arguments
                .iter()
                .any(|argument| matches!(argument, Argument::SpreadElement(_)))
        {
            return;
        }
        let bound =
            simple_callee_name(&call.callee).is_some_and(|raw| self.is_builtin_bind_callee(&raw));
        let arguments = call
            .arguments
            .iter()
            .skip(usize::from(bound))
            .map(|argument| match argument {
                Argument::SpreadElement(spread) => {
                    ArgumentValue::Spread(self.value_expression(&spread.argument))
                }
                other => ArgumentValue::Value(
                    other.as_expression().map_or_else(Vec::new, |expression| {
                        super::escaped_call_arguments::argument_values(self, expression)
                    }),
                ),
            })
            .collect();
        self.receiver_escapes.pending_possible.push(Pending {
            span: (call.span.start, call.span.end),
            arguments,
        });
    }
    pub(in super::super::super) fn finish_possible_arguments(&mut self) {
        let mut pending = std::mem::take(&mut self.receiver_escapes.pending_possible);
        if pending.is_empty() {
            return;
        }
        let mut owners: FxHashMap<(u32, u32), Vec<CallableId>> = fx_map();
        for call in &self.value_flow.calls {
            if let Some(owner) =
                super::super::parameter_call_origins::callable_owner(self, &call.callee)
            {
                owners
                    .entry((call.offset, call.end))
                    .or_default()
                    .push(owner);
            }
        }
        for (start, end, owner, _, _) in &self.receiver_escapes.methods {
            owners.entry((*start, *end)).or_default().push(*owner);
        }
        pending.sort_by_key(|call| call.span);
        let mut replay = ReplayIndex::new(self);
        for call in pending {
            let Some(owners) = owners.get_mut(&call.span) else {
                continue;
            };
            owners.sort_unstable();
            owners.dedup();
            self.project_possible_arguments(call, owners, &mut replay);
        }
    }
    fn project_possible_arguments(
        &mut self,
        call: Pending,
        owners: &[CallableId],
        replay: &mut ReplayIndex,
    ) {
        let (mut position, mut open) = (0, None);
        let mut extras: FxHashMap<usize, Vec<ValueExpression>> = fx_map();
        let mut shifted = Vec::new();
        for argument in call.arguments {
            match argument {
                ArgumentValue::Spread(value) => {
                    let already_open = open.is_some();
                    let recipient = Recipient {
                        owners: owners.to_vec(),
                        position,
                        open,
                    };
                    if let Some(projection) = replay.query(self, &value, call.span.0, recipient) {
                        if projection.has_alternatives
                            || projection.minimum != projection.baseline_length
                        {
                            if already_open {
                                shifted.extend(
                                    projection.additional.iter().map(|(_, value)| value.clone()),
                                );
                            }
                            for (slot, value) in projection.additional {
                                extras.entry(position + slot).or_default().push(value);
                            }
                            open = Some(open.map_or(position + projection.minimum, |at: usize| {
                                at.min(position + projection.minimum)
                            }));
                        }
                        if open.is_some() {
                            shifted
                                .extend(projection.baseline.iter().map(|(_, value)| value.clone()));
                        }
                        position += projection.baseline_length;
                    }
                }
                ArgumentValue::Value(values) => {
                    if open.is_some() {
                        shifted.extend(values);
                    }
                    position += 1;
                }
            }
        }
        if !extras.is_empty() {
            self.receiver_escapes
                .possible_escaped
                .insert(call.span, extras);
        }
        if let Some(start) = open.filter(|_| !shifted.is_empty()) {
            self.receiver_escapes
                .possible_open
                .insert(call.span, (start, shifted));
        }
    }
}
