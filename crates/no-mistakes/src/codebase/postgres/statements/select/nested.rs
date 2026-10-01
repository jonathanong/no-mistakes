use super::super::SqlSelectFact;
use sqlparser::ast::{Expr, GroupByExpr, Select, SelectItem};

pub(super) fn collect(
    sql: &str,
    select: &Select,
    ctes: &[String],
    in_insert_select: bool,
    out: &mut Vec<SqlSelectFact>,
) {
    walk_optional(sql, select.selection.as_ref(), ctes, in_insert_select, out);
    walk_optional(sql, select.having.as_ref(), ctes, in_insert_select, out);
    for item in &select.projection {
        if let SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } = item {
            walk_expr(sql, expr, ctes, in_insert_select, out);
        }
    }
    if let GroupByExpr::Expressions(exprs, _) = &select.group_by {
        for expr in exprs {
            walk_expr(sql, expr, ctes, in_insert_select, out);
        }
    }
    for table in &select.from {
        for join in &table.joins {
            if let Some(expr) = super::join_expr(&join.join_operator) {
                walk_expr(sql, expr, ctes, in_insert_select, out);
            }
        }
    }
}

pub(super) fn walk_expr(
    sql: &str,
    expr: &Expr,
    ctes: &[String],
    in_insert_select: bool,
    out: &mut Vec<SqlSelectFact>,
) {
    match expr {
        Expr::Exists { subquery, .. } => {
            super::collect_query(sql, subquery, ctes, in_insert_select, true, out);
        }
        Expr::Subquery(subquery) => {
            super::collect_query(sql, subquery, ctes, in_insert_select, false, out);
        }
        Expr::InSubquery { expr, subquery, .. } => {
            walk_expr(sql, expr, ctes, in_insert_select, out);
            super::collect_query(sql, subquery, ctes, in_insert_select, false, out);
        }
        Expr::AnyOp { left, right, .. } | Expr::AllOp { left, right, .. } => {
            walk_expr(sql, left, ctes, in_insert_select, out);
            walk_expr(sql, right, ctes, in_insert_select, out);
        }
        other => crate::codebase::postgres::idents::visit_child_exprs(other, &mut |child| {
            walk_expr(sql, child, ctes, in_insert_select, out);
        }),
    }
}

fn walk_optional(
    sql: &str,
    expr: Option<&Expr>,
    ctes: &[String],
    in_insert_select: bool,
    out: &mut Vec<SqlSelectFact>,
) {
    if let Some(expr) = expr {
        walk_expr(sql, expr, ctes, in_insert_select, out);
    }
}
