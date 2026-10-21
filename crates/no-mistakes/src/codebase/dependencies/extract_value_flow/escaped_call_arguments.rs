use super::*;

impl ImportCollector {
    pub(in super::super::super) fn note_escaped_call(&mut self, call: &CallExpression<'_>) {
        self.note_class_method(call);
        self.note_possible_arguments(call);
        let mut extras: FxHashMap<usize, Vec<ValueExpression>> = fx_map();
        let (mut open_at, mut open_vals, mut position): (
            Option<usize>,
            Vec<ValueExpression>,
            usize,
        ) = (None, Vec::new(), 0);
        for (raw_index, argument) in call.arguments.iter().enumerate() {
            match argument {
                Argument::SpreadElement(spread) => {
                    walk_spread_argument(
                        self,
                        &spread.argument,
                        call.span.start,
                        &mut position,
                        &mut open_at,
                        &mut extras,
                        &mut open_vals,
                    );
                }
                other => {
                    let values = other
                        .as_expression()
                        .map_or_else(Vec::new, |expression| argument_values(self, expression));
                    if open_at.is_some() {
                        open_vals.extend(values);
                    } else if values.len() != 1 || position != raw_index {
                        if !values.is_empty() {
                            extras.entry(position).or_default().extend(values);
                        }
                        position += 1;
                    } else {
                        position += 1;
                    }
                }
            }
        }
        if !extras.is_empty() {
            self.receiver_escapes
                .escaped
                .insert((call.span.start, call.span.end), extras);
        }
        if let Some(start) = open_at.filter(|_| !open_vals.is_empty()) {
            self.receiver_escapes
                .open
                .insert((call.span.start, call.span.end), (start, open_vals));
        }
    }
}

/// A literal is walked so a conditional element or a nested spread is not one
/// opaque value. Named arrays stay on [`super::array_entries`].
fn walk_spread_argument(
    collector: &ImportCollector,
    expression: &Expression<'_>,
    before: u32,
    position: &mut usize,
    open_at: &mut Option<usize>,
    extras: &mut FxHashMap<usize, Vec<ValueExpression>>,
    open_vals: &mut Vec<ValueExpression>,
) {
    let expression = crate::codebase::ts_source::unwrap_ts_wrappers(expression);
    if let Expression::ArrayExpression(array) = expression {
        for element in &array.elements {
            match element {
                oxc_ast::ast::ArrayExpressionElement::SpreadElement(nested) => {
                    // The nested spread has its own elements. Opening the tail
                    // here would drop them.
                    walk_spread_argument(
                        collector,
                        &nested.argument,
                        before,
                        position,
                        open_at,
                        extras,
                        open_vals,
                    );
                }
                oxc_ast::ast::ArrayExpressionElement::Elision(_) => {
                    if open_at.is_none() {
                        *position += 1;
                    }
                }
                other => {
                    let values = other.as_expression().map_or_else(Vec::new, |expression| {
                        argument_values(collector, expression)
                    });
                    if open_at.is_some() {
                        open_vals.extend(values);
                    } else {
                        extras.entry(*position).or_default().extend(values);
                        *position += 1;
                    }
                }
            }
        }
        return;
    }
    let value = collector.value_expression(expression);
    match super::escaped_array_entries::array_entries(collector, &value, before) {
        Some(entries) if open_at.is_none() => {
            let mut next = *position;
            for (slot, value) in entries {
                let at = *position + slot;
                extras.entry(at).or_default().push(value);
                next = next.max(at + 1);
            }
            *position = next;
        }
        Some(entries) => open_vals.extend(entries.into_iter().map(|(_, value)| value)),
        None => {
            open_at.get_or_insert(*position);
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
