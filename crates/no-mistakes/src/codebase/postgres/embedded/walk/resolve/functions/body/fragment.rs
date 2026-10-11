use super::super::super::chain;
use super::super::provenance::RecoveredSql;
use super::super::shadows_param;
use super::BodySource;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::FxHashMap;
use oxc_ast::ast::{CallExpression, Expression, ExpressionStatement};

pub(super) fn apply_append_statement(
    statement: &ExpressionStatement<'_>,
    context: &BodySource<'_, '_>,
    depth: u8,
    lookup: &mut impl FnMut(&CallExpression<'_>, &str, u8) -> Option<RecoveredSql>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
    locals: &mut FxHashMap<String, RecoveredSql>,
) -> Option<()> {
    let Expression::CallExpression(call) = unwrap_ts_wrappers(&statement.expression) else {
        return None;
    };
    let (name, appended) = local_append_parts(call, context, depth, lookup, is_shadowed, locals)?;
    let sql = locals.get_mut(&name)?;
    sql.append(appended);
    Some(())
}

fn local_append_parts(
    call: &CallExpression<'_>,
    context: &BodySource<'_, '_>,
    depth: u8,
    lookup: &mut impl FnMut(&CallExpression<'_>, &str, u8) -> Option<RecoveredSql>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
    locals: &FxHashMap<String, RecoveredSql>,
) -> Option<(String, RecoveredSql)> {
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
        return None;
    };
    if member.property.name != "append" {
        return None;
    }
    let Expression::Identifier(ident) = unwrap_ts_wrappers(&member.object) else {
        return None;
    };
    let name = ident.name.to_string();
    if !locals.contains_key(&name) {
        return None;
    }
    let expr = call.arguments.first()?.as_expression()?;
    let appended = resolve_fragment(expr, context, depth, lookup, is_shadowed, locals)?;
    Some((name, appended))
}

pub(super) fn resolve_fragment(
    expr: &Expression<'_>,
    context: &BodySource<'_, '_>,
    depth: u8,
    lookup: &mut impl FnMut(&CallExpression<'_>, &str, u8) -> Option<RecoveredSql>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
    locals: &FxHashMap<String, RecoveredSql>,
) -> Option<RecoveredSql> {
    match unwrap_ts_wrappers(expr) {
        Expression::Identifier(ident) => {
            if shadows_param(context.resolvable, ident.name.as_str()) {
                return None;
            }
            locals.get(ident.name.as_str()).cloned()
        }
        _ => chain::resolve_expr(
            expr,
            context.source,
            depth,
            lookup,
            is_shadowed,
            context.imported_sql_tags,
        ),
    }
}
