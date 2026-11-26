use super::Binding;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    AssignmentTarget, CallExpression, Expression, IdentifierReference, Program, SpreadElement,
    StaticMemberExpression,
};
use oxc_ast_visit::{walk, Visit};
use std::collections::BTreeMap;

pub(super) fn remove_unsound(program: &Program<'_>, bindings: &mut BTreeMap<String, Binding>) {
    let mut scan = Scan { bindings };
    scan.visit_program(program);
}

struct Scan<'a> {
    bindings: &'a mut BTreeMap<String, Binding>,
}

impl Scan<'_> {
    fn tracked_ident<'a>(&self, expr: &'a Expression<'a>) -> Option<&'a str> {
        let Expression::Identifier(identifier) = unwrap_ts_wrappers(expr) else {
            return None;
        };
        self.bindings
            .contains_key(identifier.name.as_str())
            .then_some(identifier.name.as_str())
    }

    fn taint(&mut self, name: &str) {
        self.bindings.remove(name);
    }

    fn taint_target(&mut self, target: &AssignmentTarget<'_>) {
        match target {
            AssignmentTarget::AssignmentTargetIdentifier(identifier) => {
                self.taint(identifier.name.as_str());
            }
            AssignmentTarget::ComputedMemberExpression(member) => {
                if let Some(name) = self.tracked_ident(&member.object) {
                    self.taint(name);
                }
            }
            AssignmentTarget::StaticMemberExpression(member) => {
                if let Some(name) = self.tracked_ident(&member.object) {
                    self.taint(name);
                }
            }
            _ => {}
        }
    }
}

impl<'a> Visit<'a> for Scan<'_> {
    fn visit_export_named_declaration(
        &mut self,
        _export: &oxc_ast::ast::ExportNamedDeclaration<'a>,
    ) {
        // `export { name }` is a read of the binding, not a mutation or alias.
    }

    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        self.taint(identifier.name.as_str());
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
            walk::walk_call_expression(self, call);
            return;
        };
        let Some(name) = self.tracked_ident(&member.object) else {
            walk::walk_call_expression(self, call);
            return;
        };
        let name = name.to_string();
        if member.optional || member.property.name.as_str() != "join" || call.arguments.len() > 1 {
            self.taint(&name);
        }
        for argument in &call.arguments {
            self.visit_argument(argument);
        }
    }

    fn visit_static_member_expression(&mut self, member: &StaticMemberExpression<'a>) {
        if !member.optional && self.tracked_ident(&member.object).is_some() {
            return;
        }
        walk::walk_static_member_expression(self, member);
    }

    fn visit_spread_element(&mut self, spread: &SpreadElement<'a>) {
        if self.tracked_ident(&spread.argument).is_some() {
            return;
        }
        walk::walk_spread_element(self, spread);
    }

    fn visit_assignment_expression(&mut self, assignment: &oxc_ast::ast::AssignmentExpression<'a>) {
        self.taint_target(&assignment.left);
        walk::walk_assignment_expression(self, assignment);
    }
}
