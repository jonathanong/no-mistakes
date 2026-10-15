impl ImportCollector {
    fn record_replacement_assignment(&mut self, assignment: &AssignmentExpression<'_>) {
        if !self.collect_value_flow {
            return;
        }
        self.record_callable_assignment(&assignment.left, Some(&assignment.right),
            assignment.operator == oxc_syntax::operator::AssignmentOperator::Assign,
            assignment.span.end);
        let Some((name, property)) = self.replacement_target(&assignment.left) else {
            self.invalidate_unrepresented_write(&assignment.left, assignment.span.end);
            return;
        };
        let value = if assignment.operator == oxc_syntax::operator::AssignmentOperator::Assign
            && stable_replacement_rhs(&assignment.right)
        {
            self.value_expression(&assignment.right)
        } else {
            ValueExpression::Unknown
        };
        self.record_replacement_write(
            name,
            property,
            value,
            assignment.span.end,
            matches!(
                &assignment.left,
                AssignmentTarget::AssignmentTargetIdentifier(_)
            ),
        );
    }
    fn replacement_target<'a>(
        &self,
        target: &'a AssignmentTarget<'_>,
    ) -> Option<(&'a str, Option<String>)> {
        let name = match target {
            AssignmentTarget::AssignmentTargetIdentifier(identifier) => identifier.name.as_str(),
            AssignmentTarget::StaticMemberExpression(member) => {
                let Expression::Identifier(receiver) =
                    crate::codebase::ts_source::unwrap_ts_wrappers(&member.object)
                else {
                    return None;
                };
                receiver.name.as_str()
            }
            AssignmentTarget::ComputedMemberExpression(member) => {
                let Expression::Identifier(receiver) =
                    crate::codebase::ts_source::unwrap_ts_wrappers(&member.object)
                else {
                    return None;
                };
                receiver.name.as_str()
            }
            _ => return None,
        };
        if !self.collect_value_flow
            || self
                .callee_binding_scope(name)
                .is_some_and(|scope| scope != 0)
            || !self.imported_bindings.contains(name)
        {
            return None;
        }
        let property = match target {
            AssignmentTarget::StaticMemberExpression(member) => {
                Some(member.property.name.to_string())
            }
            AssignmentTarget::ComputedMemberExpression(member) => {
                extract_value_flow::write_targets::static_key(&member.expression)
            }
            _ => None,
        };
        Some((name, property))
    }
    fn record_replacement_target(
        &mut self,
        target: &AssignmentTarget<'_>,
        value: ValueExpression,
        completed_at: u32,
    ) {
        if let Some((name, property)) = self.replacement_target(target) {
            self.record_replacement_write(
                name,
                property,
                value,
                completed_at,
                matches!(target, AssignmentTarget::AssignmentTargetIdentifier(_)),
            );
        } else {
            self.invalidate_unrepresented_write(target, completed_at);
        }
    }
    fn invalidate_unrepresented_write(&mut self, target: &AssignmentTarget<'_>, completed_at: u32) {
        if !self.collect_value_flow {
            return;
        }
        if !matches!(target, AssignmentTarget::AssignmentTargetIdentifier(_)) {
            self.value_flow
                .replacements
                .get_or_insert_with(Default::default);
            self.replacement_region_index();
        }
        // A destructuring target can reset an imported receiver without a
        // direct property write. Reuse the existing syntactic target extractor
        // while collecting this unsupported write; reads must still see a reset.
        for (name, path, _) in extract_value_flow::write_targets::targets(target) {
            if path.is_empty() {
                self.record_replacement_write(
                    &name,
                    None,
                    ValueExpression::Unknown,
                    completed_at,
                    true,
                );
            }
        }
        self.invalidate_replacement_execution();
    }
    fn record_replacement_write(
        &mut self,
        name: &str,
        property: Option<String>,
        value: ValueExpression,
        completed_at: u32,
        receiver_reset: bool,
    ) {
        if !self.collect_value_flow
            || self
                .callee_binding_scope(name)
                .is_some_and(|scope| scope != 0)
            || !self.imported_bindings.contains(name)
        {
            return;
        }
        self.value_flow
            .replacements
            .get_or_insert_with(Default::default);
        let region = self.replacement_region_index();
        let write = ValueReplacementWrite {
            receiver: name.to_string(),
            property,
            receiver_reset,
            owner: self.current_function_id(),
            region,
            completed_at,
            value,
            unconditional: self.replacement_unconditional(),
        };
        let facts = self
            .value_flow
            .replacements
            .as_mut()
            .and_then(std::sync::Arc::get_mut)
            .expect("write facts");
        facts.needs_call_proofs |= write.owner.is_some() && write.region.is_none();
        facts.writes.push(write);
    }
}

fn stable_replacement_rhs(expression: &Expression<'_>) -> bool {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
        Expression::Identifier(_)
        | Expression::FunctionExpression(_)
        | Expression::ArrowFunctionExpression(_) => true,
        Expression::StaticMemberExpression(member) => matches!(
            crate::codebase::ts_source::unwrap_ts_wrappers(&member.object),
            Expression::Identifier(_)
        ),
        Expression::ComputedMemberExpression(member) => {
            matches!(
                crate::codebase::ts_source::unwrap_ts_wrappers(&member.object),
                Expression::Identifier(_)
            ) && matches!(
                crate::codebase::ts_source::unwrap_ts_wrappers(&member.expression),
                Expression::StringLiteral(_) | Expression::NumericLiteral(_)
            )
        }
        _ => false,
    }
}

include!("extract_callable_assignment.rs");
