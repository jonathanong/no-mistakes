use super::*;

impl ImportCollector {
    fn assignment_rhs_receiver_context_available(&self) -> bool {
        #[cfg(feature = "test-instrumentation")]
        ordinary_demand_metrics::ASSIGNMENT_CONTEXT_CHECKS.with(|count| count.set(count.get() + 1));
        self.receiver_root_phases.is_none() && self.fresh_receiver_effects.phases.is_none()
    }
    pub(super) fn assignment_member_value_interested(
        &self,
        assignment: &AssignmentExpression<'_>,
    ) -> Option<(usize, String, bool, bool)> {
        if !self.collect_value_flow
            || !assignment.operator.is_assign()
            || !matches!(
                assignment.left,
                AssignmentTarget::StaticMemberExpression(_)
                    | AssignmentTarget::ComputedMemberExpression(_)
            )
            || !self.assignment_rhs_receiver_context_available()
        {
            return None;
        }
        let has_receiver = self.assigned_rhs_expression_has_receiver(&assignment.right);
        let fresh_root = self.receiver_only_bindings.is_some()
            && matches!(
                crate::codebase::ts_source::unwrap_ts_wrappers(&assignment.right),
                Expression::ObjectExpression(_) | Expression::ArrayExpression(_)
            );
        if !self.prior_member_mutations.has_assigned_origins() && !has_receiver && !fresh_root {
            return None;
        }
        let (name, path, _) = write_targets::targets(&assignment.left).pop()?;
        if path.last().is_some_and(Option::is_none) && self.current_function_id().is_none() {
            return None;
        }
        let base = name.split('.').next().expect("receiver binding");
        let scope = self.callee_binding_scope(base)?;
        let version = self.receiver_only_bindings.as_ref().and_then(|_| {
            self.mutable_receiver_version(
                &(scope, base.to_string()),
                (assignment.left.span().start, self.current_function_id()),
            )
        });
        let interested =
            self.prior_member_mutations
                .assigned_origin_interested(&(scope, base), &path, version);
        // A rebound root intentionally has no ordinary fresh capture. Retain
        // only the demanded identity barrier needed by a later RHS alias.
        let barrier = fresh_root
            && version.is_some()
            && self.fresh_receiver_effects.rebound(scope, base)
            && self
                .prior_member_mutations
                .has_rhs_reference_demand(scope, base);
        (interested || has_receiver || barrier)
            .then(|| (scope, base.to_string(), interested, barrier))
    }
    pub(super) fn assignment_rhs_is_complete(
        &self,
        expression: &Expression<'_>,
        value: &ValueExpression,
    ) -> bool {
        matches!(
            value,
            ValueExpression::Object(_, _) | ValueExpression::Array(_)
        ) || self.assigned_rhs_has_receiver(value)
            || matches!(
                crate::codebase::ts_source::unwrap_ts_wrappers(expression),
                Expression::NullLiteral(_)
                    | Expression::BooleanLiteral(_)
                    | Expression::NumericLiteral(_)
                    | Expression::BigIntLiteral(_)
                    | Expression::StringLiteral(_)
                    | Expression::FunctionExpression(_)
                    | Expression::ArrowFunctionExpression(_)
            )
            || *value == ValueExpression::Undefined
    }
    pub(super) fn assignment_rhs_stable_alias(&self, value: &ValueExpression) -> bool {
        let ValueExpression::Named { name, scope, .. } = value else {
            return false;
        };
        let scope = scope.unwrap_or(0);
        if name.contains('.') || self.has_mutable_receiver_binding(scope, name) {
            return false;
        }
        if scope == 0 && self.imported_bindings.contains(name) {
            return true;
        }
        self.value_binding_indexes
            .get(&scope)
            .and_then(|names| names.get(name))
            .is_some_and(|indices| {
                indices
                    .iter()
                    .all(|index| self.value_flow.bindings[*index].invalidated_at.is_none())
                    && self.assigned_rhs_name_has_receiver(name, scope)
            })
            || self
                .value_parameter_indexes
                .get(&scope)
                .and_then(|names| names.get(name))
                .is_some_and(|indices| {
                    indices
                        .iter()
                        .all(|index| self.value_flow.parameters[*index].invalidated_at.is_none())
                })
    }
    fn assigned_rhs_expression_has_receiver(&self, expression: &Expression<'_>) -> bool {
        match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
            Expression::ObjectExpression(object) => !object.properties.is_empty(),
            Expression::ArrayExpression(array) => !array.elements.is_empty(),
            Expression::Identifier(identifier) => self.assigned_rhs_name_has_receiver(
                identifier.name.as_str(),
                self.callee_binding_scope(identifier.name.as_str())
                    .unwrap_or(0),
            ),
            Expression::StaticMemberExpression(member) => {
                self.assigned_rhs_expression_has_receiver(&member.object)
            }
            Expression::ComputedMemberExpression(member) => {
                self.assigned_rhs_expression_has_receiver(&member.object)
            }
            _ => false,
        }
    }
    fn assigned_rhs_name_has_receiver(&self, name: &str, scope: usize) -> bool {
        self.has_mutable_receiver_binding(scope, name)
            || (scope == 0 && self.imported_bindings.contains(name))
            || self
                .value_parameter_indexes
                .get(&scope)
                .is_some_and(|names| names.contains_key(name))
            || self
                .value_binding_indexes
                .get(&scope)
                .and_then(|names| names.get(name))
                .is_some_and(|indices| {
                    indices.iter().any(|index| {
                        matches!(
                            self.value_flow.bindings[*index].value,
                            ValueExpression::Object(_, _)
                                | ValueExpression::Array(_)
                                | ValueExpression::Named { .. }
                                | ValueExpression::Member(_, _)
                        )
                    })
                })
    }
    pub(super) fn assigned_rhs_has_receiver(&self, value: &ValueExpression) -> bool {
        match value {
            ValueExpression::Object(fields, _) | ValueExpression::Array(fields) => fields
                .values()
                .any(|value| self.assigned_rhs_has_receiver(value)),
            ValueExpression::Member(base, _) | ValueExpression::Awaited(base) => {
                self.assigned_rhs_has_receiver(base)
            }
            ValueExpression::Named { name, scope, .. } => {
                self.assigned_rhs_name_has_receiver(name, scope.unwrap_or(0))
            }
            _ => false,
        }
    }
}
