use super::super::super::sql_text;
use super::super::super::tags::interpolating_untrusted_tag;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Argument, BinaryOperator, CallExpression, Expression};

/// Resolves a fluent `.append()` chain (or `+` composition, or a call into
/// a same-file statically-composed function) into its SQL text.
///
/// `lookup` resolves a bare-identifier call target (a same-file function
/// name) to its own resolved body text, or `None` when it isn't one.
/// `is_shadowed` reports whether a name is currently bound to something
/// other than its global meaning (e.g. the trusted `sql` tag shadowed by a
/// same-file helper's own parameter) — see [`interpolating_untrusted_tag`].
/// `depth` bounds recursion so a cyclic or pathological chain fails closed
/// instead of overflowing the stack.
pub(super) fn resolve_expr(
    expr: &Expression<'_>,
    depth: u8,
    lookup: &mut impl FnMut(&str, u8) -> Option<String>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
) -> Option<String> {
    let depth = depth.checked_sub(1)?;
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(literal) => Some(literal.value.to_string()),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => sql_text(expr),
        Expression::TaggedTemplateExpression(_)
            if interpolating_untrusted_tag(expr, is_shadowed) =>
        {
            None
        }
        Expression::TaggedTemplateExpression(_) => sql_text(expr),
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            let left = resolve_expr(&binary.left, depth, lookup, is_shadowed)?;
            let right = resolve_expr(&binary.right, depth, lookup, is_shadowed)?;
            let right = renumber_placeholders(&right, count_placeholders(&left));
            Some(format!("{left}{right}"))
        }
        Expression::CallExpression(call) => resolve_call(call, depth, lookup, is_shadowed),
        _ => None,
    }
}

fn resolve_call(
    call: &CallExpression<'_>,
    depth: u8,
    lookup: &mut impl FnMut(&str, u8) -> Option<String>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
) -> Option<String> {
    match unwrap_ts_wrappers(&call.callee) {
        Expression::StaticMemberExpression(member) if member.property.name == "append" => {
            let base = resolve_expr(&member.object, depth, lookup, is_shadowed)?;
            let appended = resolve_expr(append_argument(call)?, depth, lookup, is_shadowed)?;
            let appended = renumber_placeholders(&appended, count_placeholders(&base));
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

const PLACEHOLDER_MARKER: &str = "sql_placeholder_";

pub(super) fn count_placeholders(text: &str) -> u32 {
    text.matches(PLACEHOLDER_MARKER).count() as u32
}

/// Each independently-resolved fragment numbers its own placeholders from 1,
/// so joining two fragments via `.append()` would otherwise duplicate
/// `sql_placeholder_1`. Shift every placeholder in `text` by `offset` (the
/// placeholder count already used by the fragment it's being joined after)
/// so the joined result stays sequential in source order.
pub(super) fn renumber_placeholders(text: &str, offset: u32) -> String {
    if offset == 0 {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut seen = 0u32;
    while let Some(position) = rest.find(PLACEHOLDER_MARKER) {
        out.push_str(&rest[..position]);
        let after_marker = &rest[position + PLACEHOLDER_MARKER.len()..];
        let digits = after_marker.bytes().take_while(u8::is_ascii_digit).count();
        seen += 1;
        out.push_str(PLACEHOLDER_MARKER);
        out.push_str(&(offset + seen).to_string());
        rest = &after_marker[digits..];
    }
    out.push_str(rest);
    out
}
