use super::*;

impl ImportCollector {
    pub(in super::super::super) fn note_escaped_call(&mut self, call: &CallExpression<'_>) {
        self.note_class_method(call);
        self.note_possible_arguments(call);
        let mut arguments = super::escaped_spread_arguments::Arguments::default();
        for (raw_index, argument) in call.arguments.iter().enumerate() {
            match argument {
                Argument::SpreadElement(spread) => {
                    arguments.spread(self, &spread.argument, call.span.start);
                }
                other => {
                    let values = other
                        .as_expression()
                        .map_or_else(Vec::new, |expression| argument_values(self, expression));
                    arguments.ordinary(values, raw_index);
                }
            }
        }
        let span = (call.span.start, call.span.end);
        if arguments.guarded_open_tail {
            if let Some(index) = call
                .arguments
                .iter()
                .position(|argument| matches!(argument, Argument::SpreadElement(_)))
            {
                let method_shift = self
                    .receiver_escapes
                    .methods
                    .last()
                    .filter(|(start, end, _, _, _)| (*start, *end) == span)
                    .map_or(0, |(_, _, _, values, _)| {
                        call.arguments.len().saturating_sub(values.len())
                    });
                let normalized_call = simple_callee_name(&call.callee).is_some_and(|raw| {
                    raw.ends_with(".call")
                        && recorded_call_callee(self, call).is_some_and(|name| name != raw)
                });
                self.receiver_escapes.uncertain_spreads.insert(
                    span,
                    UncertainSpread {
                        raw: index,
                        shift: method_shift.max(usize::from(normalized_call)),
                    },
                );
            }
        }
        if !arguments.extras.is_empty() {
            self.receiver_escapes.escaped.insert(span, arguments.extras);
        }
        if !arguments.guarded_extras.is_empty() {
            self.receiver_escapes
                .guarded_escaped
                .insert(span, arguments.guarded_extras);
        }
        if let Some(start) = arguments.open_at {
            if !arguments.open_vals.is_empty() {
                self.receiver_escapes
                    .open
                    .insert(span, (start, arguments.open_vals));
            }
            if !arguments.guarded_open.is_empty() {
                self.receiver_escapes
                    .guarded_open
                    .insert(span, (start, arguments.guarded_open));
            }
        }
    }
}

/// Both sides of a conditional or logical expression can be the runtime argument.
pub(super) fn argument_values(
    collector: &ImportCollector,
    expression: &Expression<'_>,
) -> Vec<ValueExpression> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
        Expression::ConditionalExpression(conditional) => {
            let mut values = argument_values(collector, &conditional.consequent);
            values.extend(argument_values(collector, &conditional.alternate));
            values
        }
        Expression::LogicalExpression(logical) => {
            let mut values = argument_values(collector, &logical.left);
            values.extend(argument_values(collector, &logical.right));
            values
        }
        Expression::SequenceExpression(sequence) => sequence
            .expressions
            .last()
            .map(|expression| argument_values(collector, expression))
            .unwrap_or_default(),
        other => vec![collector.value_expression(other)],
    }
}
