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
    if let Expression::AssignmentExpression(assignment) = value {
        if assignment.operator == AssignmentOperator::Assign {
            if let Some((receiver, index)) = argument_slot(&assignment.left) {
                return Expr::SlotWrite {
                    receiver: Box::new(super::expression(receiver, source)),
                    index,
                    value: Box::new(super::expression(&assignment.right, source)),
                };
            }
        }
    }
    let mut targets = Targets::default();
    if let Expression::AssignmentExpression(value) = value {
        targets.visit_assignment_target(&value.left);
    }
    if let Expression::UpdateExpression(value) = value {
        targets.visit_simple_assignment_target(&value.argument);
    }
    targets.0.sort();
    targets.0.dedup();
    let mut children = children::collect(value, source);
    if matches!(value, Expression::AssignmentExpression(assignment)
        if assignment.operator == AssignmentOperator::Assign
            && !matches!(&assignment.left, AssignmentTarget::ArrayAssignmentTarget(_) | AssignmentTarget::ObjectAssignmentTarget(_)))
        && matches!(children.first(), Some(Expr::Name(name)) if targets.0.contains(name))
    {
        // A simple binding target is not a value passed to opaque code. Its RHS
        // remains a separate child, including a read of the same binding.
        children.remove(0);
    }
    Expr::OpaqueWrite {
        children,
        targets: targets.0,
    }
}

fn argument_slot<'a, 's>(target: &'s AssignmentTarget<'a>) -> Option<(&'s Expression<'a>, usize)> {
    let AssignmentTarget::ComputedMemberExpression(member) = target else {
        return None;
    };
    if !matches!(unwrap_ts_wrappers(&member.object), Expression::Identifier(id) if id.name == "arguments")
    {
        return None;
    }
    static_index(unwrap_ts_wrappers(&member.expression)).map(|index| (&member.object, index))
}

fn static_index(value: &Expression<'_>) -> Option<usize> {
    const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;
    match unwrap_ts_wrappers(value) {
        Expression::NumericLiteral(value)
            if value.value.is_finite()
                && value.value <= MAX_SAFE_INTEGER
                && value.value.fract() == 0.0
                && (value.value as usize) as f64 == value.value =>
        {
            Some(value.value as usize)
        }
        Expression::StringLiteral(value) => value
            .value
            .parse::<usize>()
            .ok()
            .filter(|index| index.to_string() == value.value.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
