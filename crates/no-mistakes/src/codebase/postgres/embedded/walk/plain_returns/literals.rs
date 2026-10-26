use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::Expression;

pub(super) fn is_direct_function(expression: &Expression<'_>) -> bool {
    matches!(
        unwrap_ts_wrappers(expression),
        Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_)
    )
}

pub(super) fn is_plain_literal(expression: &Expression<'_>) -> bool {
    match unwrap_ts_wrappers(expression) {
        Expression::StringLiteral(_) => true,
        Expression::TemplateLiteral(template) => template.expressions.is_empty(),
        _ => false,
    }
}
