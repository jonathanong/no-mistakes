use super::*;

impl ImportCollector {
    pub(super) fn is_builtin_bind_callee(&self, raw: &str) -> bool {
        raw.strip_suffix(".bind").is_some_and(|receiver| {
            is_function_prototype_receiver(self, receiver)
                && !receiver_owns_member(self, receiver, "bind")
        })
    }

    pub(super) fn value_arguments(
        &self,
        arguments: &[Argument<'_>],
    ) -> (Vec<ValueExpression>, Option<usize>) {
        (
            arguments
                .iter()
                .map(|argument| {
                    argument
                        .as_expression()
                        .map_or(ValueExpression::Unknown, |expression| {
                            self.value_expression(expression)
                        })
                })
                .collect(),
            arguments
                .iter()
                .position(|argument| argument.as_expression().is_none()),
        )
    }
    pub(super) fn adapted_value_call(
        &self,
        call: &CallExpression<'_>,
    ) -> (ValueExpression, Vec<ValueExpression>, Option<usize>, bool) {
        if let Some(raw) = simple_callee_name(&call.callee) {
            if let Some(receiver) = raw.strip_suffix(".bind") {
                if self.is_builtin_bind_callee(&raw) {
                    let prefix = self.bound_prefix(call);
                    let (arguments, spread) = self.value_arguments(&call.arguments[prefix.start..]);
                    let spread = if prefix.uncertain {
                        Some(0)
                    } else {
                        spread.map(|index| index + prefix.leading.len())
                    };
                    let mut leading = prefix.leading;
                    leading.extend(arguments);
                    return (
                        ValueExpression::Named {
                            name: receiver.to_string(),
                            scope: self.callee_binding_scope(receiver),
                            offset: call.span.start,
                            context: self.current_function_id(),
                        },
                        leading,
                        spread,
                        true,
                    );
                }
            }
        }
        let raw = simple_callee_name(&call.callee);
        let normalized = recorded_call_callee(self, call);
        if let (Some(raw), Some(normalized)) = (raw, normalized) {
            if raw != normalized && function_prototype_call_apply_receiver(self, &raw).is_some() {
                let callee = ValueExpression::Named {
                    scope: self.callee_binding_scope(&normalized),
                    name: normalized,
                    offset: call.span.start,
                    context: self.current_function_id(),
                };
                if raw.ends_with(".call") {
                    let (arguments, spread) = self.value_arguments(&call.arguments);
                    let shifted = arguments.into_iter().skip(1).collect();
                    return (
                        callee,
                        shifted,
                        spread.map(|index| index.saturating_sub(1)),
                        false,
                    );
                }
                let array = call.arguments.get(1).and_then(Argument::as_expression);
                return match array {
                    None | Some(Expression::NullLiteral(_)) => (callee, Vec::new(), None, false),
                    Some(expression)
                        if self.value_expression(expression) == ValueExpression::Undefined =>
                    {
                        (callee, Vec::new(), None, false)
                    }
                    Some(Expression::ArrayExpression(array)) => {
                        let arguments = array
                            .elements
                            .iter()
                            .map(|element| {
                                if matches!(
                                    element,
                                    oxc_ast::ast::ArrayExpressionElement::Elision(_)
                                ) {
                                    return ValueExpression::Undefined;
                                }
                                element
                                    .as_expression()
                                    .map_or(ValueExpression::Unknown, |value| {
                                        self.value_expression(value)
                                    })
                            })
                            .collect();
                        let spread = array.elements.iter().position(|element| {
                            matches!(
                                element,
                                oxc_ast::ast::ArrayExpressionElement::SpreadElement(_)
                            )
                        });
                        (callee, arguments, spread, false)
                    }
                    _ => (callee, Vec::new(), Some(0), false),
                };
            }
        }
        let (arguments, spread) = self.value_arguments(&call.arguments);
        (
            self.value_expression(&call.callee),
            arguments,
            spread,
            false,
        )
    }
    pub(in super::super) fn record_value_new(&mut self, new: &NewExpression<'_>) {
        if !self.collect_value_flow {
            return;
        }
        let (arguments, spread_at) = self.value_arguments(&new.arguments);
        let value_call = ValueCall {
            context: self.current_function_id(),
            offset: new.span.start,
            end: new.span.end,
            source_callee: simple_callee_name(&new.callee)
                .unwrap_or_else(|| "<unknown>".to_string()),
            callee: self.value_expression(&new.callee),
            arguments: std::sync::Arc::new(arguments),
            bound: false,
            spread_at,
            once: false,
            repeated: self.repeat_depth > self.call_repeat_base,
        };
        self.note_array_push(&value_call, false);
        self.value_flow.calls.push(value_call);
    }
}
