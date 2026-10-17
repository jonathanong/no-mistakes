use super::*;

#[cfg(feature = "test-instrumentation")]
thread_local! {
    pub(super) static DISABLED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
impl ImportCollector {
    pub(super) fn record_single_named_root(
        &mut self,
        scope: usize,
        name: &str,
        assignment: &AssignmentExpression<'_>,
    ) -> bool {
        #[cfg(feature = "test-instrumentation")]
        if DISABLED.with(std::cell::Cell::get) {
            return false;
        }
        // Phase receipts still own their full value events. Keep that path intact.
        if self.receiver_root_phases.is_some()
            || self.fresh_receiver_effects.phases.is_some()
            || self.destructuring_phase_owner.is_some()
        {
            return false;
        }
        let Expression::ObjectExpression(object) = &assignment.right else {
            return false;
        };
        let [ObjectPropertyKind::ObjectProperty(property)] = object.properties.as_slice() else {
            return false;
        };
        if property.computed || property.method || property.kind != PropertyKind::Init {
            return false;
        }
        let Some(field) = crate::codebase::ts_source::static_property_key_name(&property.key)
        else {
            return false;
        };
        if field == "__proto__" && !property.shorthand {
            return false;
        }
        let Expression::Identifier(identifier) = &property.value else {
            return false;
        };
        // The ordinary expression converter decides whether undefined is global.
        if identifier.name == "undefined" {
            return false;
        }
        let value_scope = self.callee_binding_scope(identifier.name.as_str());
        let read = (identifier.span.start, self.current_function_id());
        let Some(origins) = &mut self.receiver_only_bindings else {
            return false;
        };
        let value = origins.single_named_field(field, identifier.name.as_str(), value_scope, read);
        self.fresh_receiver_effects.rebind(scope, name);
        self.retain_receiver_origin(
            scope,
            name,
            value,
            assignment.span.end,
            !assignment.operator.is_logical(),
        );
        true
    }
}
