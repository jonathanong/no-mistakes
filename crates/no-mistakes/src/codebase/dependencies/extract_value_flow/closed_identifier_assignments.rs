use super::*;
use oxc_ast::ast::{AssignmentExpression, AssignmentTarget, VariableDeclarator};

impl ImportCollector {
    pub(crate) fn note_identifier_initializer(&mut self, declarator: &VariableDeclarator<'_>) {
        if !self.collect_identifier_provenance {
            return;
        }
        let BindingPattern::BindingIdentifier(identifier) = &declarator.id else {
            return;
        };
        let name = identifier.name.as_str();
        let scope = self
            .callee_binding_scope(name)
            .unwrap_or(self.current_lexical_scope_id());
        let value = if self
            .unreachable_value_binding_after
            .is_some_and(|at| declarator.span.start >= at)
        {
            ValueExpression::Unknown
        } else {
            declarator
                .init
                .as_ref()
                .map_or(ValueExpression::Undefined, |init| {
                    self.identifier_assignment_expression(init)
                })
        };
        let owner = self.current_function_id();
        self.identifier_initializers
            .entry(scope)
            .or_default()
            .insert(
                name.to_string(),
                IdentifierInitializer {
                    declared_at: declarator.span.start,
                    owner,
                    value,
                },
            );
    }

    pub(crate) fn identifier_assignment(
        &self,
        assignment: &AssignmentExpression<'_>,
    ) -> Option<Box<IdentifierAssignment>> {
        if !self.collect_identifier_provenance || !assignment.operator.is_assign() {
            return None;
        }
        let AssignmentTarget::AssignmentTargetIdentifier(target) = &assignment.left else {
            return None;
        };
        let scope = self.callee_binding_scope(target.name.as_str())?;
        if scope == 0 && self.imported_bindings.contains(target.name.as_str()) {
            return None;
        }
        Some(Box::new(IdentifierAssignment {
            completed_at: assignment.span.end,
            owner: self.current_function_id(),
            unconditional: !self.conditional_execution
                && !self.receiver_write_is_conditional
                && self.replacement_region.is_none(),
            value: self.identifier_assignment_expression(&assignment.right),
            initial: None,
        }))
    }

    fn identifier_assignment_expression(&self, expression: &Expression<'_>) -> ValueExpression {
        use std::sync::Arc;
        let expression = crate::codebase::ts_source::unwrap_ts_wrappers(expression);
        match expression {
            Expression::ObjectExpression(_)
            | Expression::ArrayExpression(_)
            | Expression::JSXElement(_)
            | Expression::JSXFragment(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::FunctionExpression(_) => ValueExpression::Unknown,
            Expression::TemplateLiteral(template) => ValueExpression::Derived(
                template
                    .expressions
                    .iter()
                    .map(|value| self.identifier_assignment_expression(value))
                    .collect::<Vec<_>>()
                    .into(),
            ),
            Expression::BinaryExpression(binary)
                if binary.operator == oxc_syntax::operator::BinaryOperator::Addition =>
            {
                ValueExpression::Derived(
                    vec![
                        self.identifier_assignment_expression(&binary.left),
                        self.identifier_assignment_expression(&binary.right),
                    ]
                    .into(),
                )
            }
            Expression::ConditionalExpression(value) => ValueExpression::Either(
                Arc::new(self.identifier_assignment_expression(&value.consequent)),
                Arc::new(self.identifier_assignment_expression(&value.alternate)),
            ),
            Expression::LogicalExpression(value) => ValueExpression::Either(
                Arc::new(self.identifier_assignment_expression(&value.left)),
                Arc::new(self.identifier_assignment_expression(&value.right)),
            ),
            Expression::AwaitExpression(value) => ValueExpression::Awaited(Arc::new(
                self.identifier_assignment_expression(&value.argument),
            )),
            Expression::StaticMemberExpression(value) => ValueExpression::Member(
                Arc::new(self.identifier_assignment_expression(&value.object)),
                value.property.name.to_string(),
            ),
            Expression::ComputedMemberExpression(value) => super::super::write_targets::static_key(
                &value.expression,
            )
            .map_or(ValueExpression::Unknown, |key| {
                ValueExpression::Member(
                    Arc::new(self.identifier_assignment_expression(&value.object)),
                    key,
                )
            }),
            Expression::SequenceExpression(value) => value
                .expressions
                .last()
                .map_or(ValueExpression::Unknown, |value| {
                    self.identifier_assignment_expression(value)
                }),
            _ => self.value_expression(expression),
        }
    }

    pub(crate) fn note_identifier_assignment(
        &mut self,
        assignment: &AssignmentExpression<'_>,
        start: usize,
        value: Option<Box<IdentifierAssignment>>,
    ) {
        let Some(mut value) = value else { return };
        if let Some(write) = self.value_flow.identifier_writes[start..]
            .iter_mut()
            .find(|write| write.at == assignment.span.start && write.members.is_empty())
        {
            value.initial = self
                .identifier_initializers
                .get_mut(&write.scope)
                .and_then(|names| names.remove(write.name.as_str()))
                .map(Box::new);
            write.assigned = Some(value);
        }
    }
}

#[cfg(test)]
#[path = "identifier_provenance_demand_tests.rs"]
mod tests;
