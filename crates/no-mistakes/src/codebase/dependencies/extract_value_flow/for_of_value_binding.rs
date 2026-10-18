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
        if values.is_empty() {
            return;
        }
        // Each literal element can become the loop binding on a later pass.
        let mut value = values.pop().expect("nonempty literal array");
        for next in values {
            value = ValueExpression::Default(std::sync::Arc::new(value), std::sync::Arc::new(next));
        }
        self.record_value_pattern(&declarator.id, value, declarator.span.start);
    }
}
