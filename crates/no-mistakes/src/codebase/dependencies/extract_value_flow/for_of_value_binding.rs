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
        let Some(declarator) = declaration.declarations.first() else {
            return;
        };
        let ValueExpression::Array(elements) = self.value_expression(right) else {
            return;
        };
        let mut entries: Vec<_> = elements.iter().collect();
        entries.sort_by(|left, right| left.0.cmp(right.0));
        let mut values: Vec<_> = entries
            .into_iter()
            .map(|(_, value)| value.clone())
            .collect();
        if !values
            .iter()
            .any(|value| self.assigned_rhs_has_receiver(value))
        {
            return;
        }
        let BindingPattern::BindingIdentifier(identifier) = &declarator.id else {
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
        self.retain_receiver_origin(scope, name, value, declarator.span.end, false);
    }
}
