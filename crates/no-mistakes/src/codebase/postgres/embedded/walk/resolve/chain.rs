use super::super::super::sql_text;
use super::super::super::tags::interpolating_untrusted_tag;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Argument, BinaryOperator, CallExpression, Expression};

/// Resolves a fluent `.append()` chain (or `+` composition, or a call into
/// a same-file statically-composed function) into its SQL text.
///
/// `lookup` resolves a bare-identifier call target (a same-file function
/// name) to its own resolved body text, or `None` when it isn't one.
/// `depth` bounds recursion so a cyclic or pathological chain fails closed
/// instead of overflowing the stack.
pub(super) fn resolve_expr(
    expr: &Expression<'_>,
    depth: u8,
    lookup: &mut impl FnMut(&str, u8) -> Option<String>,
) -> Option<String> {
    let depth = depth.checked_sub(1)?;
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(literal) => Some(literal.value.to_string()),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => sql_text(expr),
        Expression::TaggedTemplateExpression(_) if interpolating_untrusted_tag(expr) => None,
        Expression::TaggedTemplateExpression(_) => sql_text(expr),
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            let left = resolve_expr(&binary.left, depth, lookup)?;
            let right = resolve_expr(&binary.right, depth, lookup)?;
            Some(format!("{left}{right}"))
        }
        Expression::CallExpression(call) => resolve_call(call, depth, lookup),
        _ => None,
    }
}

fn resolve_call(
    call: &CallExpression<'_>,
    depth: u8,
    lookup: &mut impl FnMut(&str, u8) -> Option<String>,
) -> Option<String> {
    match unwrap_ts_wrappers(&call.callee) {
        Expression::StaticMemberExpression(member) if member.property.name == "append" => {
            let base = resolve_expr(&member.object, depth, lookup)?;
            let appended = resolve_expr(append_argument(call)?, depth, lookup)?;
            Some(format!("{base}{appended}"))
        }
        Expression::Identifier(ident) => lookup(ident.name.as_str(), depth),
        _ => None,
    }
}

fn append_argument<'a>(call: &'a CallExpression<'a>) -> Option<&'a Expression<'a>> {
    match call.arguments.first()? {
        Argument::SpreadElement(_) => None,
        other => other.as_expression(),
    }
}
