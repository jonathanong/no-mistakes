use super::super::super::tags::{interpolating_untrusted_tag, SqlTagNames};
use super::super::super::unpublished_sql_text;
use super::functions::provenance::RecoveredSql;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{BinaryOperator, CallExpression, Expression};

mod helpers;
use helpers::{append_argument, is_identifier};

/// Resolves a fluent `.append()` chain (or `+` composition, or a call into
/// a same-file statically-composed function) into its SQL text.
///
/// `lookup` resolves a bare-identifier call target (a same-file function
/// name) to its own resolved body text, or `None` when it isn't one.
/// `is_shadowed` reports whether a name is currently bound to something
/// other than its global meaning (e.g. the trusted `sql` tag shadowed by a
/// same-file helper's own parameter) — see [`interpolating_untrusted_tag`].
/// `imported_sql_tags` are local names of a default import from
/// `sql-template-strings`, or of a configured named SQL tag.
/// `depth` bounds recursion so a cyclic or pathological chain fails closed
/// instead of overflowing the stack.
pub(super) fn resolve_expr(
    expr: &Expression<'_>,
    source: &str,
    depth: u8,
    lookup: &mut impl FnMut(&CallExpression<'_>, &str, u8) -> Option<RecoveredSql>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
    idents: &mut impl FnMut(&str) -> Option<RecoveredSql>,
    imported_sql_tags: &SqlTagNames,
) -> Option<RecoveredSql> {
    let depth = depth.checked_sub(1)?;
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(literal) => Some(RecoveredSql::literal(
            expr,
            source,
            crate::ast::js_literal(&literal).into_owned(),
        )),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => Some(
            RecoveredSql::literal(expr, source, unpublished_sql_text(expr)?),
        ),
        Expression::TaggedTemplateExpression(_) => Some(RecoveredSql::literal(
            expr,
            source,
            tagged_sql(expr, is_shadowed, imported_sql_tags)?,
        )),
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            let mut left = resolve_expr(
                &binary.left,
                source,
                depth,
                lookup,
                is_shadowed,
                idents,
                imported_sql_tags,
            )?;
            left.append(resolve_expr(
                &binary.right,
                source,
                depth,
                lookup,
                is_shadowed,
                idents,
                imported_sql_tags,
            )?);
            Some(left)
        }
        Expression::CallExpression(call) => resolve_call(
            call,
            source,
            depth,
            lookup,
            is_shadowed,
            idents,
            imported_sql_tags,
        ),
        Expression::Identifier(ident) => idents(ident.name.as_str()),
        _ => None,
    }
}

/// Recovers an already-selected template without a call-target lookup.
pub(super) fn tagged_sql(
    expr: &Expression<'_>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
    imported_sql_tags: &SqlTagNames,
) -> Option<String> {
    if interpolating_untrusted_tag(expr, is_shadowed, imported_sql_tags) {
        None
    } else {
        unpublished_sql_text(expr)
    }
}

/// Recovers a verified leading statement from a fluent `.append()` chain
/// whose later composition is opaque. The caller must classify this as
/// dynamic rather than treating the returned prefix as complete SQL.
pub(super) fn resolve_dynamic_prefix(
    expr: &Expression<'_>,
    source: &str,
    depth: u8,
    lookup: &mut impl FnMut(&CallExpression<'_>, &str, u8) -> Option<RecoveredSql>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
    idents: &mut impl FnMut(&str) -> Option<RecoveredSql>,
    imported_sql_tags: &SqlTagNames,
) -> Option<RecoveredSql> {
    let depth = depth.checked_sub(1)?;
    let Expression::CallExpression(call) = unwrap_ts_wrappers(expr) else {
        return None;
    };
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
        return None;
    };
    if member.property.name != "append" || is_identifier(&member.object) {
        return None;
    }
    let base = resolve_expr(
        &member.object,
        source,
        depth,
        lookup,
        is_shadowed,
        idents,
        imported_sql_tags,
    )
    .or_else(|| {
        resolve_dynamic_prefix(
            &member.object,
            source,
            depth,
            lookup,
            is_shadowed,
            idents,
            imported_sql_tags,
        )
    })?;
    super::super::scope::has_known_leading_statement(&base.sql).then_some(base)
}

fn resolve_call(
    call: &CallExpression<'_>,
    source: &str,
    depth: u8,
    lookup: &mut impl FnMut(&CallExpression<'_>, &str, u8) -> Option<RecoveredSql>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
    idents: &mut impl FnMut(&str) -> Option<RecoveredSql>,
    imported_sql_tags: &SqlTagNames,
) -> Option<RecoveredSql> {
    match unwrap_ts_wrappers(&call.callee) {
        Expression::StaticMemberExpression(member)
            if member.property.name == "append" && !is_identifier(&member.object) =>
        {
            let mut arguments = vec![append_argument(call)?];
            let mut receiver = &member.object;
            while let Expression::CallExpression(inner) = unwrap_ts_wrappers(receiver) {
                let Expression::StaticMemberExpression(link) = unwrap_ts_wrappers(&inner.callee)
                else {
                    break;
                };
                if link.property.name != "append" {
                    break;
                }
                arguments.push(append_argument(inner)?);
                receiver = &link.object;
            }
            if is_identifier(receiver) {
                return None;
            }
            let mut base = resolve_expr(
                receiver,
                source,
                depth,
                lookup,
                is_shadowed,
                idents,
                imported_sql_tags,
            )?;
            for argument in arguments.into_iter().rev() {
                let appended = resolve_expr(
                    argument,
                    source,
                    depth,
                    lookup,
                    is_shadowed,
                    idents,
                    imported_sql_tags,
                )?;
                base.append(appended);
            }
            Some(base)
        }
        Expression::Identifier(ident) => lookup(call, ident.name.as_str(), depth),
        _ => None,
    }
}
