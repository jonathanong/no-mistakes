use super::super::ScopeVisitor;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::Expression;

/// Follow only result-producing paths: a helper used as a condition does not
/// make the chosen result an alias of that helper's mutable builder.
pub(super) fn contains(visitor: &ScopeVisitor<'_>, expr: &Expression<'_>) -> bool {
    let mut pending = vec![expr];
    while let Some(expr) = pending.pop() {
        match unwrap_ts_wrappers(expr) {
            Expression::ConditionalExpression(branch) => {
                pending.extend([&branch.consequent, &branch.alternate]);
            }
            Expression::LogicalExpression(branch) => {
                pending.extend([&branch.left, &branch.right]);
            }
            Expression::CallExpression(call) => {
                if super::super::resolve::compose::contains_parameter_helper(expr, visitor) {
                    return true;
                }
                if let Expression::ComputedMemberExpression(member) =
                    unwrap_ts_wrappers(&call.callee)
                {
                    if matches!(unwrap_ts_wrappers(&member.expression), Expression::StringLiteral(property) if property.value == "append")
                    {
                        pending.push(&member.object);
                    }
                }
            }
            _ => {}
        }
    }
    false
}
