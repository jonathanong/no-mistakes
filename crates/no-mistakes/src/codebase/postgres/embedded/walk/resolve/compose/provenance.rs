use super::{functions, resolve_chain_recovered, ScopeVisitor};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{BinaryOperator, Expression};

pub(in crate::codebase::postgres::embedded::walk) fn helper_provenance(
    expr: &Expression<'_>,
    visitor: &ScopeVisitor<'_>,
) -> Option<functions::provenance::RecoveredSql> {
    if visitor.head_calls_only {
        return None;
    }
    fn contains_helper(expr: &Expression<'_>, visitor: &ScopeVisitor<'_>) -> bool {
        match unwrap_ts_wrappers(expr) {
            Expression::CallExpression(call) => match unwrap_ts_wrappers(&call.callee) {
                Expression::Identifier(name) => {
                    visitor.functions.has_static_helper(name.name.as_str())
                        || visitor.is_hole_helper(name.name.as_str())
                }
                Expression::StaticMemberExpression(member) if member.property.name == "append" => {
                    contains_helper(&member.object, visitor)
                }
                _ => false,
            },
            Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
                contains_helper(&binary.left, visitor) || contains_helper(&binary.right, visitor)
            }
            _ => false,
        }
    }
    contains_helper(expr, visitor)
        .then(|| resolve_chain_recovered(expr, visitor))
        .flatten()
}
