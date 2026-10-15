use super::*;

impl ImportCollector {
    pub(in super::super) fn record_callable_value_use(&mut self, expression: &Expression<'_>) {
        if !self.collect_value_flow {
            return;
        }
        let Expression::Identifier(identifier) = expression else {
            return;
        };
        if self.bare_callable_callee == Some(identifier.span.start) {
            return;
        }
        self.record_callable_name_value_use(identifier.name.as_str());
    }

    pub(in super::super) fn record_callable_name_value_use(&mut self, name: &str) {
        if !self.collect_value_flow {
            return;
        }
        let name = name.split('.').next().unwrap_or(name);
        if !self.has_local_function_scope(name) {
            return;
        }
        let scope = self
            .callee_binding_scope(name)
            .expect("local callable binding scope");
        let uses = self
            .value_flow
            .callable_value_uses
            .get_or_insert_with(|| std::sync::Arc::new(fx_map()));
        std::sync::Arc::make_mut(uses)
            .entry(scope)
            .or_default()
            .insert(name.to_string());
    }
}
