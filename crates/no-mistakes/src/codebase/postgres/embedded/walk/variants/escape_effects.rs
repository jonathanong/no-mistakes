use super::super::ScopeVisitor;
use crate::codebase::postgres::embedded::bindings::callee_name;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    Argument, ArrayExpressionElement, AssignmentOperator, AssignmentTarget, CallExpression,
    Expression, ObjectPropertyKind,
};
use oxc_ast_visit::Visit;

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn visit_variant_guard<'a>(
        &mut self,
        expression: &Expression<'a>,
    ) where
        Self: Visit<'a>,
    {
        let previous = self
            .variant_guard_owner
            .replace(self.function_scopes.last().copied().unwrap_or(0));
        self.visit_expression(expression);
        self.variant_guard_owner = previous;
    }

    pub(super) fn escaped_builder_effect(
        &self,
        call: &CallExpression<'_>,
    ) -> Vec<(String, Option<u32>)> {
        if self.known_builder_consumer(call) {
            return Vec::new();
        }
        let mut pending = call
            .arguments
            .iter()
            .filter_map(|argument| match argument {
                Argument::SpreadElement(spread) => Some(&spread.argument),
                argument => argument.as_expression(),
            })
            .collect::<Vec<_>>();
        let mut effects = Vec::new();
        while let Some(expression) = pending.pop() {
            match unwrap_ts_wrappers(expression) {
                Expression::ArrayExpression(array) => {
                    pending.extend(array.elements.iter().filter_map(|element| match element {
                        ArrayExpressionElement::SpreadElement(spread) => Some(&spread.argument),
                        element => element.as_expression(),
                    }));
                }
                Expression::ObjectExpression(object) => {
                    pending.extend(object.properties.iter().map(|property| match property {
                        ObjectPropertyKind::ObjectProperty(property) => &property.value,
                        ObjectPropertyKind::SpreadProperty(spread) => &spread.argument,
                    }));
                }
                Expression::ConditionalExpression(branch) => {
                    pending.extend([&branch.consequent, &branch.alternate]);
                }
                Expression::LogicalExpression(branch) => {
                    pending.extend([&branch.left, &branch.right]);
                }
                Expression::SequenceExpression(sequence) => {
                    pending.extend(sequence.expressions.last());
                }
                Expression::AssignmentExpression(assignment)
                    if assignment.operator == AssignmentOperator::Assign =>
                {
                    if let AssignmentTarget::AssignmentTargetIdentifier(id) = &assignment.left {
                        if let Some(identity) = self.builder_identity_for_init(&assignment.right) {
                            effects.push((id.name.to_string(), Some(identity)));
                        }
                    }
                    pending.push(&assignment.right);
                }
                _ => {
                    effects.extend(
                        self.builder_result_names(expression)
                            .into_iter()
                            .filter_map(|name| {
                                let binding = self.lookup(&name)?;
                                // SQL strings are copied by value. Only a tracked mutable
                                // statement object can escape through an unknown call.
                                (binding.initialized
                                    && (binding.builder_identity.is_some()
                                        || binding.variants.as_ref().is_some_and(|values| {
                                            values.iter().any(|value| value.fragment)
                                        })))
                                .then_some((name, binding.builder_identity))
                            }),
                    );
                }
            }
        }
        effects
    }

    pub(super) fn known_builder_consumer(&self, call: &CallExpression<'_>) -> bool {
        let callee = unwrap_ts_wrappers(&call.callee);
        if callee_name(call, self.bindings, self.scoped).is_some()
            || self.fresh_builder_call(call)
            || self.known_append_receiver(callee)
        {
            return true;
        }
        // Relative executors are confirmed after this same extractor pass.
        matches!(callee, Expression::Identifier(name) if self.provisional.iter().any(|span|
            span.name == name.name.as_str() && call.span.start >= span.start
                && call.span.start < span.end && !span.owners.is_empty()))
    }

    fn known_append_receiver(&self, callee: &Expression<'_>) -> bool {
        let receiver = match callee {
            Expression::StaticMemberExpression(member) if member.property.name == "append" => {
                &member.object
            }
            Expression::ComputedMemberExpression(member) if matches!(unwrap_ts_wrappers(&member.expression), Expression::StringLiteral(name) if name.value == "append") => {
                &member.object
            }
            _ => return false,
        };
        self.builder_identity_for_init(receiver).is_some()
            || self.builder_result_names(receiver).iter().any(|name| {
                self.lookup(name).is_some_and(|binding| {
                    !binding.initialized && binding.sql_builder
                        || binding
                            .variants
                            .as_ref()
                            .is_some_and(|values| values.iter().any(|value| value.fragment))
                })
            })
    }
}
