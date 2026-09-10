use super::super::super::placeholders::{count_placeholders, renumber_placeholders};
use super::super::super::tags::interpolating_untrusted_tag;
use super::super::super::unpublished_sql_text;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Argument, BinaryOperator, CallExpression, Expression};
use std::collections::HashSet;

/// Resolves a fluent `.append()` chain (or `+` composition, or a call into
/// a same-file statically-composed function) into its SQL text.
///
/// `lookup` resolves a bare-identifier call target (a same-file function
/// name) to its own resolved body text, or `None` when it isn't one.
/// `is_shadowed` reports whether a name is currently bound to something
/// other than its global meaning (e.g. the trusted `sql` tag shadowed by a
/// same-file helper's own parameter) — see [`interpolating_untrusted_tag`].
/// `imported_sql_tags` are local names of a default import from
/// `sql-template-strings`, which is the trusted tag under any spelling.
/// `depth` bounds recursion so a cyclic or pathological chain fails closed
/// instead of overflowing the stack.
pub(super) fn resolve_expr(
    expr: &Expression<'_>,
    depth: u8,
    lookup: &mut impl FnMut(&str, u8) -> Option<String>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
    imported_sql_tags: &HashSet<String>,
) -> Option<String> {
    let depth = depth.checked_sub(1)?;
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(literal) => Some(literal.value.to_string()),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            unpublished_sql_text(expr)
        }
        Expression::TaggedTemplateExpression(_)
            if interpolating_untrusted_tag(expr, is_shadowed, imported_sql_tags) =>
        {
            None
        }
        Expression::TaggedTemplateExpression(_) => unpublished_sql_text(expr),
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            let left = resolve_expr(&binary.left, depth, lookup, is_shadowed, imported_sql_tags)?;
            let right = resolve_expr(&binary.right, depth, lookup, is_shadowed, imported_sql_tags)?;
            let right = renumber_placeholders(&right, count_placeholders(&left));
            Some(format!("{left}{right}"))
        }
        Expression::CallExpression(call) => {
            resolve_call(call, depth, lookup, is_shadowed, imported_sql_tags)
        }
        _ => None,
    }
}

/// Recovers a verified leading statement from a fluent `.append()` chain
/// whose later composition is opaque. The caller must classify this as
/// dynamic rather than treating the returned prefix as complete SQL.
pub(super) fn resolve_dynamic_prefix(
    expr: &Expression<'_>,
    depth: u8,
    lookup: &mut impl FnMut(&str, u8) -> Option<String>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
    imported_sql_tags: &HashSet<String>,
) -> Option<String> {
    let depth = depth.checked_sub(1)?;
    let Expression::CallExpression(call) = unwrap_ts_wrappers(expr) else {
        return None;
    };
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
        return None;
    };
    if member.property.name != "append" {
        return None;
    }
    let base = resolve_expr(
        &member.object,
        depth,
        lookup,
        is_shadowed,
        imported_sql_tags,
    )
    .or_else(|| {
        resolve_dynamic_prefix(
            &member.object,
            depth,
            lookup,
            is_shadowed,
            imported_sql_tags,
        )
    })?;
    super::super::scope::has_known_leading_statement(&base).then_some(base)
}

fn resolve_call(
    call: &CallExpression<'_>,
    depth: u8,
    lookup: &mut impl FnMut(&str, u8) -> Option<String>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
    imported_sql_tags: &HashSet<String>,
) -> Option<String> {
    match unwrap_ts_wrappers(&call.callee) {
        Expression::StaticMemberExpression(member) if member.property.name == "append" => {
            let base = resolve_expr(
                &member.object,
                depth,
                lookup,
                is_shadowed,
                imported_sql_tags,
            )?;
            let appended = resolve_expr(
                append_argument(call)?,
                depth,
                lookup,
                is_shadowed,
                imported_sql_tags,
            )?;
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
