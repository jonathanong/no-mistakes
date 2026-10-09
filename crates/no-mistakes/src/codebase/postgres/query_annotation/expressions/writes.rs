//! Opaque writes retain explicit binding targets separately from value reads.
use super::{children, Expr};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;

#[derive(Default)]
struct Targets(Vec<String>);
impl Targets {
    fn wrapped(&mut self, value: &Expression<'_>) {
        if let Expression::Identifier(id) = unwrap_ts_wrappers(value) {
            self.0.push(id.name.to_string());
        }
    }
}
impl<'a> Visit<'a> for Targets {
    fn visit_identifier_reference(&mut self, value: &IdentifierReference<'a>) {
        self.0.push(value.name.to_string());
    }
    // Receivers, computed keys, and destructuring defaults are value reads.
    fn visit_static_member_expression(&mut self, _: &StaticMemberExpression<'a>) {}
    fn visit_computed_member_expression(&mut self, _: &ComputedMemberExpression<'a>) {}
    fn visit_private_field_expression(&mut self, _: &PrivateFieldExpression<'a>) {}
    fn visit_expression(&mut self, _: &Expression<'a>) {}
    fn visit_ts_as_expression(&mut self, value: &TSAsExpression<'a>) {
        self.wrapped(&value.expression);
    }
    fn visit_ts_satisfies_expression(&mut self, value: &TSSatisfiesExpression<'a>) {
        self.wrapped(&value.expression);
    }
    fn visit_ts_non_null_expression(&mut self, value: &TSNonNullExpression<'a>) {
        self.wrapped(&value.expression);
    }
    fn visit_ts_type_assertion(&mut self, value: &TSTypeAssertion<'a>) {
        self.wrapped(&value.expression);
    }
}

pub(super) fn collect(value: &Expression<'_>, source: &str) -> Expr {
    let value = unwrap_ts_wrappers(value);
    let mut targets = Targets::default();
    if let Expression::AssignmentExpression(value) = value {
        targets.visit_assignment_target(&value.left);
    }
    if let Expression::UpdateExpression(value) = value {
        targets.visit_simple_assignment_target(&value.argument);
    }
    targets.0.sort();
    targets.0.dedup();
    Expr::OpaqueWrite {
        children: children::collect(value, source),
        targets: targets.0,
    }
}

#[cfg(test)]
mod tests;
