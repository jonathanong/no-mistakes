use super::super::super::super::super::placeholders::{count_placeholders, renumber_placeholders};
use super::super::super::chain;
use super::super::{shadows_param, Resolvable};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression, ExpressionStatement};
use std::collections::{HashMap, HashSet};

pub(super) fn apply_append_statement(
    statement: &ExpressionStatement<'_>,
    resolvable: &Resolvable<'_>,
    depth: u8,
    lookup: &mut impl FnMut(&str, u8) -> Option<String>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
    imported_sql_tags: &HashSet<String>,
    locals: &mut HashMap<String, String>,
) -> Option<()> {
    let Expression::CallExpression(call) = unwrap_ts_wrappers(&statement.expression) else {
        return None;
    };
    let (name, appended) = local_append_parts(
        call,
        resolvable,
        depth,
        lookup,
        is_shadowed,
        imported_sql_tags,
        locals,
    )?;
    let sql = locals.get_mut(&name)?;
    let appended = renumber_placeholders(&appended, count_placeholders(sql));
    sql.push_str(&appended);
    Some(())
}

fn local_append_parts(
    call: &CallExpression<'_>,
    resolvable: &Resolvable<'_>,
    depth: u8,
    lookup: &mut impl FnMut(&str, u8) -> Option<String>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
    imported_sql_tags: &HashSet<String>,
    locals: &HashMap<String, String>,
) -> Option<(String, String)> {
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
    let appended = resolve_fragment(
        expr,
        resolvable,
        depth,
        lookup,
        is_shadowed,
        imported_sql_tags,
        locals,
    )?;
    Some((name, appended))
}

pub(super) fn resolve_fragment(
    expr: &Expression<'_>,
    resolvable: &Resolvable<'_>,
    depth: u8,
    lookup: &mut impl FnMut(&str, u8) -> Option<String>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
    imported_sql_tags: &HashSet<String>,
    locals: &HashMap<String, String>,
) -> Option<String> {
    match unwrap_ts_wrappers(expr) {
        Expression::Identifier(ident) => {
            if shadows_param(resolvable, ident.name.as_str()) {
                return None;
            }
            locals.get(ident.name.as_str()).cloned()
        }
        _ => chain::resolve_expr(expr, depth, lookup, is_shadowed, imported_sql_tags),
    }
}
