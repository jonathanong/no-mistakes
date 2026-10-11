use super::super::ScopeVisitor;
use super::ValueKind;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    ArrowFunctionExpression, AssignmentExpression, AssignmentTarget, CallExpression, Expression,
    Function, SimpleAssignmentTarget, UpdateExpression,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;

impl ScopeVisitor<'_> {
    pub(super) fn expression_mutates_builder(&self, expression: &Expression<'_>) -> bool {
        let mut effects = Effects {
            visitor: self,
            found: false,
        };
        effects.visit_expression(expression);
        effects.found
    }
}

struct Effects<'visitor, 'source> {
    visitor: &'visitor ScopeVisitor<'source>,
    found: bool,
}

impl Effects<'_, '_> {
    fn known_sql_binding(&self, name: &str) -> bool {
        self.visitor.lookup(name).is_some_and(|binding| {
            binding.sql_builder
                || binding.builder_identity.is_some()
                || binding
                    .variants
                    .as_ref()
                    .is_some_and(|values| values.iter().any(|value| value.value == ValueKind::Sql))
        })
    }
    fn appends_existing_builder(&self, call: &CallExpression<'_>) -> bool {
        let receiver = match unwrap_ts_wrappers(&call.callee) {
            Expression::StaticMemberExpression(member) if member.property.name == "append" => {
                &member.object
            }
            Expression::ComputedMemberExpression(member) if matches!(unwrap_ts_wrappers(&member.expression), Expression::StringLiteral(property) if property.value == "append") => {
                &member.object
            }
            _ => return false,
        };
        self.visitor
            .builder_result_names(receiver)
            .iter()
            .any(|name| self.known_sql_binding(name))
    }
}

impl<'a> Visit<'a> for Effects<'_, '_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.found |= self
            .visitor
            .parameter_helper_effect(call)
            .is_some_and(|effects| !effects.is_empty())
            || self.appends_existing_builder(call);
        if !self.found {
            walk::walk_call_expression(self, call);
        }
    }
    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        if let AssignmentTarget::AssignmentTargetIdentifier(id) = &assignment.left {
            self.found |= self.known_sql_binding(id.name.as_str());
        }
        if !self.found {
            walk::walk_assignment_expression(self, assignment);
        }
    }
    fn visit_update_expression(&mut self, update: &UpdateExpression<'a>) {
        if let SimpleAssignmentTarget::AssignmentTargetIdentifier(id) = &update.argument {
            self.found |= self.known_sql_binding(id.name.as_str());
        }
        if !self.found {
            walk::walk_update_expression(self, update);
        }
    }
    // Creating a callback does not execute mutations inside its body.
    fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _: &ArrowFunctionExpression<'a>) {}
}
