use super::*;

impl ImportCollector {
    pub(in super::super) fn record_default_value(&mut self, export: &ExportDefaultDeclaration<'_>) {
        if !self.collect_value_flow {
            return;
        }
        let Some(expression) = export.declaration.as_expression() else {
            return;
        };
        if matches!(expression, Expression::Identifier(_)) {
            return;
        }
        let value = self.value_expression(expression);
        self.value_binding_indexes
            .entry(0)
            .or_default()
            .entry("default".to_string())
            .or_default()
            .push(self.value_flow.bindings.len());
        self.record_value_reference(0, "default", &value);
        self.value_flow.bindings.push(ValueBinding {
            immutable: true,
            owner: None,
            name: "default".to_string(),
            scope: 0,
            declared_at: export.span.start,
            repeated: false,
            invalidated_at: None,
            mutation_contexts: fx_set(),
            value,
        });
    }
}
