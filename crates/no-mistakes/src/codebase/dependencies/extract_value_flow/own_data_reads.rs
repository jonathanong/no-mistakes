use super::*;

impl ImportCollector {
    pub(super) fn known_own_data_read(
        &self,
        expression: &Expression<'_>,
        allow_mutable: bool,
    ) -> bool {
        let Expression::StaticMemberExpression(member) = expression else {
            return false;
        };
        let Expression::Identifier(root) =
            crate::codebase::ts_source::unwrap_ts_wrappers(&member.object)
        else {
            return false;
        };
        let name = root.name.as_str();
        let Some(scope) = self.callee_binding_scope(name) else {
            return false;
        };
        if self.fresh_receiver_effects.rebound(scope, name) {
            return false;
        }
        let declared_at = if let Some(indices) = self
            .value_binding_indexes
            .get(&scope)
            .and_then(|names| names.get(name))
        {
            let [index] = indices.as_slice() else {
                return false;
            };
            let binding = &self.value_flow.bindings[*index];
            if binding.owner != self.current_function_id()
                || !matches!(&binding.value, ValueExpression::Object(fields, _)
                    if fields.contains_key(member.property.name.as_str()))
            {
                return false;
            }
            binding.declared_at
        } else {
            if !allow_mutable {
                return false;
            }
            let owner = self.current_function_id();
            // Preparation may follow physically later LHS writes. Its caller
            // separately rejects roots changed before this execution phase.
            let offset = self
                .receiver_root_phases
                .as_ref()
                .filter(|capture| capture.owner == owner)
                .map_or(member.span.end, |capture| {
                    capture.catalog.start.saturating_sub(1)
                });
            let Some(declared_at) = self.receiver_only_bindings.as_ref().and_then(|origins| {
                origins.own_data_member(scope, name, (offset, owner), member.property.name.as_str())
            }) else {
                return false;
            };
            declared_at
        };
        self.fresh_receiver_effects.epoch == 0
            && !self
                .fresh_receiver_effects
                .descriptor_call
                .is_some_and(|offset| offset >= declared_at)
    }
}
