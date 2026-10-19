use super::*;

impl ImportCollector {
    pub(in super::super::super) fn record_for_of_value_binding(
        &mut self,
        left: &ForStatementLeft<'_>,
        right: &Expression<'_>,
    ) {
        if !self.collect_value_flow {
            return;
        }
        let ForStatementLeft::VariableDeclaration(declaration) = left else {
            return;
        };
        // A named array (`const items = [box]`) is the same receiver list as a
        // literal. One binding hop; anything else is not a known element list.
        let Some(mut values) = super::super::parameter_call_origins::array_elements(
            self,
            &self.value_expression(right),
        ) else {
            return;
        };
        if !values
            .iter()
            .any(|value| self.assigned_rhs_has_receiver(value))
        {
            return;
        }
        let Some((identifier, declaration_end)) =
            declaration
                .declarations
                .first()
                .and_then(|declarator| match &declarator.id {
                    BindingPattern::BindingIdentifier(identifier) => {
                        Some((identifier, declarator.span.end))
                    }
                    _ => None,
                })
        else {
            return;
        };
        // Receiver histories traverse both Default branches as alternatives.
        // Ordinary bindings interpret Default as fallback, so keep loop values
        // in receiver-only history even when the array has one element.
        let mut value = values.pop().expect("nonempty literal array");
        for next in values {
            value = ValueExpression::Default(std::sync::Arc::new(value), std::sync::Arc::new(next));
        }
        let name = identifier.name.as_str();
        let scope = self
            .callee_binding_scope(name)
            .unwrap_or(self.current_lexical_scope_id());
        self.capture_mutable_root_phase((scope, name), identifier.span.start, &value, false);
        self.retain_receiver_origin(scope, name, value, declaration_end, false);
    }
}
