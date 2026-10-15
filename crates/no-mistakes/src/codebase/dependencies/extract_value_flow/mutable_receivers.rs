use super::*;

impl ImportCollector {
    pub(super) fn record_mutable_receiver_binding(
        &mut self,
        pattern: &BindingPattern<'_>,
        value: ValueExpression,
        completed_at: u32,
    ) {
        let BindingPattern::BindingIdentifier(identifier) = pattern else {
            return;
        };
        if !matches!(
            value,
            ValueExpression::Object(..) | ValueExpression::Array(..)
        ) {
            return;
        }
        let name = identifier.name.as_str();
        let scope = self
            .callee_binding_scope(name)
            .unwrap_or(self.current_lexical_scope_id());
        self.capture_mutable_root_phase((scope, name), identifier.span.start, &value, true);
        self.retain_receiver_origin(scope, name, value, completed_at, true);
    }

    pub(in super::super) fn prepare_mutable_pattern_roots(
        &mut self,
        target: &AssignmentTarget<'_>,
    ) {
        if self.receiver_only_bindings.is_none()
            || !matches!(
                target,
                AssignmentTarget::ArrayAssignmentTarget(_)
                    | AssignmentTarget::ObjectAssignmentTarget(_)
            )
        {
            return;
        }
        if write_targets::targets(target)
            .iter()
            .any(|(name, path, _)| {
                path.is_empty()
                    && self
                        .callee_binding_scope(name)
                        .is_some_and(|scope| self.has_mutable_receiver_binding(scope, name))
            })
        {
            self.prepare_value_references();
        }
    }

    pub(in super::super) fn record_mutable_receiver_assignment(
        &mut self,
        assignment: &AssignmentExpression<'_>,
    ) {
        if self.receiver_only_bindings.is_none() {
            return;
        }
        if let Some(name) = mutable_root_name(&assignment.left) {
            if let Some(scope) = self.callee_binding_scope(name) {
                if self.has_mutable_receiver_binding(scope, name) {
                    let value = self.value_expression(&assignment.right);
                    self.capture_mutable_root_phase(
                        (scope, name),
                        assignment.left.span().start,
                        &value,
                        !assignment.operator.is_logical(),
                    );
                    self.fresh_receiver_effects.rebind(scope, name);
                    self.retain_receiver_origin(
                        scope,
                        name,
                        value,
                        assignment.span.end,
                        !assignment.operator.is_logical(),
                    );
                }
            }
        } else if matches!(
            assignment.left,
            AssignmentTarget::ArrayAssignmentTarget(_)
                | AssignmentTarget::ObjectAssignmentTarget(_)
        ) {
            // Destructuring replaces lexical roots too. Its projected RHS is
            // intentionally unknown here; never keep a stale prior root.
            #[cfg(feature = "test-instrumentation")]
            {
                self.value_flow.receiver_root_target_decompositions += 1;
            }
            for (name, path, target) in write_targets::targets(&assignment.left) {
                if path.is_empty() {
                    if let Some(scope) = self.callee_binding_scope(&name) {
                        if self.has_mutable_receiver_binding(scope, &name) {
                            self.fresh_receiver_effects.rebind(scope, &name);
                            self.capture_mutable_root_phase(
                                (scope, &name),
                                target,
                                &ValueExpression::Unknown,
                                true,
                            );
                            self.retain_receiver_origin(
                                scope,
                                &name,
                                ValueExpression::Unknown,
                                assignment.span.end,
                                true,
                            );
                        }
                    }
                }
            }
        }
    }

    pub(in super::super) fn has_mutable_receiver_binding(&self, scope: usize, name: &str) -> bool {
        self.receiver_only_bindings
            .as_ref()
            .is_some_and(|origins| origins.contains(scope, name))
    }

    pub(super) fn references_mutable_receiver(&self, value: &ValueExpression) -> bool {
        matches!(value, ValueExpression::Named { name, scope, .. }
            if !name.contains('.') && self.has_mutable_receiver_binding(scope.unwrap_or(0), name))
    }

    pub(super) fn mutable_receiver_version(
        &self,
        key: &(usize, String),
        read: nested::RootRead,
    ) -> Option<mutable_origins::RootVersion> {
        self.receiver_only_bindings
            .as_ref()
            .and_then(|origins| origins.version(key, read))
    }

    pub(super) fn mutable_receiver_origin_watermark(&self) -> usize {
        self.receiver_only_bindings
            .as_ref()
            .map_or(0, |origins| origins.watermark())
    }

    pub(super) fn append_mutable_receiver_references_unchecked<R: nested::RootCoordinate>(
        &self,
        key: &(usize, String),
        coordinates: (&[Option<String>], &[nested::ReceiverRead], nested::RootRead),
        output: &mut impl Extend<nested::ReceiverWrite<R>>,
        foreign_before: usize,
    ) {
        if let Some(origins) = &self.receiver_only_bindings {
            origins.append_references(key, coordinates, output, foreign_before);
        }
    }

    fn retain_receiver_origin(
        &mut self,
        scope: usize,
        name: &str,
        value: ValueExpression,
        completed_at: u32,
        unconditional: bool,
    ) {
        if self.value_reference_links.is_some() {
            self.prior_member_mutations
                .require_reference_proof(scope, name);
        }
        let guaranteed = unconditional
            && !self.receiver_write_is_conditional
            && self.repeat_depth == self.call_repeat_base;
        let owner = self.current_function_id();
        self.receiver_only_bindings
            .get_or_insert_with(Default::default)
            .record((scope, name), (completed_at, owner), value, guaranteed);
        #[cfg(feature = "test-instrumentation")]
        {
            self.value_flow.receiver_only_origins += 1;
        }
    }
}

fn mutable_root_name<'a>(target: &'a AssignmentTarget<'_>) -> Option<&'a str> {
    let expression = match target {
        AssignmentTarget::AssignmentTargetIdentifier(identifier) => {
            return Some(identifier.name.as_str())
        }
        AssignmentTarget::TSAsExpression(value) => &value.expression,
        AssignmentTarget::TSSatisfiesExpression(value) => &value.expression,
        AssignmentTarget::TSNonNullExpression(value) => &value.expression,
        AssignmentTarget::TSTypeAssertion(value) => &value.expression,
        _ => return None,
    };
    if let Expression::Identifier(identifier) =
        crate::codebase::ts_source::unwrap_ts_wrappers(expression)
    {
        Some(identifier.name.as_str())
    } else {
        None
    }
}
