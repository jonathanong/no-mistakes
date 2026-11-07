use super::*;

impl ImportCollector {
    pub(in super::super::super) fn note_sql_fragment_member(
        &mut self,
        object: &Expression<'_>,
        property: Option<&str>,
    ) {
        if !self.collect_value_flow || matches!(property, Some("text" | "values")) {
            return;
        }
        let Some(name) = names::identifier_name(object) else {
            return;
        };
        let Some(scope) = self.callee_binding_scope(&name) else {
            return;
        };
        if self.value_flow.bindings.iter().any(|binding| {
            binding.name == name
                && binding.scope == scope
                && matches!(binding.value, ValueExpression::Call(start, end)
                    if self.value_flow.tagged_templates.contains_key(&(start, end)))
        }) {
            self.record_registry_mutation(&name);
        }
    }
}
