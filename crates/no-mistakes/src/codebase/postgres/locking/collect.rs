use super::relations::locked_tables;
use super::{
    has_for_update, locks_skip_locked, order_keys, set_expr_has_multi_row, unwrap_expr,
    LockingSelectMetadata,
};
use sqlparser::ast::{Expr, Query, SetExpr, Statement, TableFactor, TableWithJoins};

pub(super) fn collect_from_statement(
    statement: &Statement,
    out: &mut Vec<LockingSelectMetadata>,
    positions: &[(u32, u32)],
) {
    if let Statement::Query(query) = statement {
        collect_from_query(query, out, positions);
    }
}

pub(super) fn collect_from_query(
    query: &Query,
    out: &mut Vec<LockingSelectMetadata>,
    positions: &[(u32, u32)],
) {
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            collect_from_query(&cte.query, out, positions);
        }
    }
    collect_from_set_expr(&query.body, out, positions);
    if has_for_update(&query.locks) {
        let locked_tables = locked_tables(&query.body, &query.locks, positions);
        out.push(LockingSelectMetadata {
            has_multi_row_predicate: set_expr_has_multi_row(&query.body),
            has_order_by: query.order_by.is_some(),
            skips_locked_rows: locks_skip_locked(&query.locks),
            tables: locked_tables.as_ref().map(|tables| tables.names.clone()),
            pinned_columns: locked_tables.as_ref().map(|tables| tables.pinned.clone()),
            join_equalities: locked_tables
                .as_ref()
                .map_or_else(Vec::new, |tables| tables.joins.clone()),
            table_qualifiers: locked_tables.map(|tables| tables.qualifiers),
            order: query.order_by.as_ref().and_then(order_keys),
        });
    }
}

pub(super) fn collect_from_set_expr(
    expr: &SetExpr,
    out: &mut Vec<LockingSelectMetadata>,
    positions: &[(u32, u32)],
) {
    match expr {
        SetExpr::Select(select) => {
            if let Some(selection) = &select.selection {
                collect_queries_from_expr(selection, out, positions);
            }
            for table in &select.from {
                collect_from_table_with_joins(table, out, positions);
            }
        }
        SetExpr::Query(query) => collect_from_query(query, out, positions),
        SetExpr::SetOperation { left, right, .. } => {
            collect_from_set_expr(left, out, positions);
            collect_from_set_expr(right, out, positions);
        }
        _ => {}
    }
}

pub(super) fn collect_from_table_with_joins(
    table: &TableWithJoins,
    out: &mut Vec<LockingSelectMetadata>,
    positions: &[(u32, u32)],
) {
    collect_from_table_factor(&table.relation, out, positions);
    for join in &table.joins {
        collect_from_table_factor(&join.relation, out, positions);
    }
}

pub(super) fn collect_from_table_factor(
    factor: &TableFactor,
    out: &mut Vec<LockingSelectMetadata>,
    positions: &[(u32, u32)],
) {
    if let TableFactor::Derived { subquery, .. } = factor {
        collect_from_query(subquery, out, positions);
    }
}

pub(super) fn collect_queries_from_expr(
    expr: &Expr,
    out: &mut Vec<LockingSelectMetadata>,
    positions: &[(u32, u32)],
) {
    match unwrap_expr(expr) {
        Expr::Subquery(query)
        | Expr::InSubquery {
            subquery: query, ..
        } => {
            collect_from_query(query, out, positions);
        }
        Expr::BinaryOp { left, right, .. } => {
            collect_queries_from_expr(left, out, positions);
            collect_queries_from_expr(right, out, positions);
        }
        Expr::UnaryOp { expr, .. } => collect_queries_from_expr(expr, out, positions),
        _ => {}
    }
}
