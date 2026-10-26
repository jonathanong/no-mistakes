use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Argument, CallExpression, Expression};

pub(super) fn append_argument<'a>(call: &'a CallExpression<'a>) -> Option<&'a Expression<'a>> {
    match call.arguments.first()? {
        Argument::SpreadElement(_) => None,
        other => other.as_expression(),
    }
}

/// `name.append(..)` mutates the builder `name` in place and returns it, so a
/// bare name is never a copy to resolve as the chain's base.
pub(super) fn is_identifier(expr: &Expression<'_>) -> bool {
    matches!(unwrap_ts_wrappers(expr), Expression::Identifier(_))
}
