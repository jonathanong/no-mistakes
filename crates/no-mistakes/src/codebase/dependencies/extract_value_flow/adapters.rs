use super::*;

impl ImportCollector {
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
    ) -> (ValueExpression, Vec<ValueExpression>, Option<usize>) {
        let raw = simple_callee_name(&call.callee);
        let normalized = recorded_call_callee(self, call);
        if let (Some(raw), Some(normalized)) = (raw, normalized) {
            if raw != normalized && function_prototype_call_apply_receiver(self, &raw).is_some() {
                let callee = ValueExpression::Named {
                    scope: self.callee_binding_scope(&normalized),
                    name: normalized,
                    offset: call.span.start,
                };
                if raw.ends_with(".call") {
                    let (arguments, spread) = self.value_arguments(&call.arguments);
                    let shifted = arguments.into_iter().skip(1).collect();
                    return (callee, shifted, spread.map(|index| index.saturating_sub(1)));
                }
                let array = call.arguments.get(1).and_then(Argument::as_expression);
                return match array {
                    None | Some(Expression::NullLiteral(_)) => (callee, Vec::new(), None),
                    Some(Expression::ArrayExpression(array)) => {
                        let arguments = array
                            .elements
                            .iter()
                            .map(|element| {
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
                        (callee, arguments, spread)
                    }
                    _ => (callee, Vec::new(), Some(0)),
                };
            }
        }
        let (arguments, spread) = self.value_arguments(&call.arguments);
        (self.value_expression(&call.callee), arguments, spread)
    }
    pub(in super::super) fn record_value_new(&mut self, new: &NewExpression<'_>) {
        if !self.collect_value_flow {
            return;
        }
        let (arguments, spread_at) = self.value_arguments(&new.arguments);
        self.value_flow.calls.push(ValueCall {
            offset: new.span.start,
            end: new.span.end,
            source_callee: simple_callee_name(&new.callee)
                .unwrap_or_else(|| "<unknown>".to_string()),
            callee: self.value_expression(&new.callee),
            arguments,
            spread_at,
        });
    }
}
