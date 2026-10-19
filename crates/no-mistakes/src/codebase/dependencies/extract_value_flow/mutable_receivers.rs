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
        ) && !self.assigned_rhs_has_receiver(&value)
        {
            return;
        }
        // Mutable aliases of tracked roots need their own rebind history; const
        // captures use the existing immutable binding path instead.
        let name = identifier.name.as_str();
        let scope = self
            .callee_binding_scope(name)
            .unwrap_or(self.current_lexical_scope_id());
        self.capture_mutable_root_phase((scope, name), identifier.span.start, &value, true);
        self.retain_receiver_origin(scope, name, value, completed_at, true);
    }

    pub(in super::super) fn record_mutable_receiver_assignment(
        &mut self,
        assignment: &AssignmentExpression<'_>,
    ) {
        self.note_parameter_reassignment(assignment);
        if self.receiver_only_bindings.is_none() {
            return;
        }
        if let Some(name) = mutable_root_name(&assignment.left) {
            if let Some(scope) = self.callee_binding_scope(name) {
                if self.has_mutable_receiver_binding(scope, name) {
                    if self.record_single_named_root(scope, name, assignment) {
                        return;
                    }
                    let value = self.root_value_expression(&assignment.right);
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
            // Select receiver identities from the same RHS facts used by this
            // assignment; phase points retain each target's actual completion.
            #[cfg(feature = "test-instrumentation")]
            {
                self.value_flow.receiver_root_target_decompositions += 1;
            }
            let mut has_root = false;
            root_targets::for_each_root(&assignment.left, |name, _| {
                has_root = has_root
                    || self
                        .callee_binding_scope(name)
                        .is_some_and(|scope| self.has_mutable_receiver_binding(scope, name));
            });
            if !has_root {
                return;
            }
            let value = self.value_expression(&assignment.right);
            for (name, target, value) in
                assignment_root_values::roots(&assignment.left, value, &mut |expression| {
                    self.value_expression(expression)
                })
            {
                if let Some(scope) = self.callee_binding_scope(&name) {
                    if self.has_mutable_receiver_binding(scope, &name) {
                        self.fresh_receiver_effects.rebind(scope, &name);
                        self.capture_mutable_root_phase((scope, &name), target, &value, true);
                        self.retain_receiver_origin(scope, &name, value, assignment.span.end, true);
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
        version: mutable_origins::RootVersion,
        writer: Option<CallableId>,
    ) {
        if let Some(origins) = &self.receiver_only_bindings {
            let defer_module = writer.is_some()
                && writer == coordinates.2 .1
                && self.active_module_provider.is_none()
                && !self.foreign_receiver_writes_finishing();
            origins.append_references(
                key,
                coordinates,
                output,
                foreign_before,
                version,
                defer_module,
            );
        }
    }

    fn note_parameter_reassignment(&mut self, assignment: &AssignmentExpression<'_>) {
        if !self.collect_value_flow || !assignment.operator.is_assign() {
            return;
        }
        let Some(name) = mutable_root_name(&assignment.left) else {
            return;
        };
        let Some(scope) = self.callee_binding_scope(name) else {
            return;
        };
        let bare = self
            .value_parameter_indexes
            .get(&scope)
            .and_then(|names| names.get(name))
            .is_some_and(|indexes| {
                indexes.iter().any(|index| {
                    self.value_flow
                        .parameters
                        .get(*index)
                        .is_some_and(|parameter| parameter.projection.is_empty())
                })
            });
        if !bare {
            return;
        }
        self.parameter_reassignments.push((
            scope,
            name.to_string(),
            self.value_expression(&assignment.right),
        ));
    }

    pub(super) fn retain_receiver_origin(
        &mut self,
        scope: usize,
        name: &str,
        value: impl Into<mutable_origins::Origin>,
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

pub(super) fn mutable_root_name<'a>(target: &'a AssignmentTarget<'_>) -> Option<&'a str> {
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
