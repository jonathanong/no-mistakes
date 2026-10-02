use super::SqlSelectFact;
use sqlparser::ast::{Expr, TableFactor};

pub(super) fn walk_factor(
    sql: &str,
    factor: &TableFactor,
    ctes: &[String],
    selects: &mut Vec<SqlSelectFact>,
) {
    match factor {
        TableFactor::Derived { subquery, .. } => {
            super::super::select::collect_query(sql, subquery, ctes, false, false, selects);
        }
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => super::walk_side_queries(
            sql,
            std::slice::from_ref(table_with_joins),
            None,
            ctes,
            selects,
        ),
        TableFactor::UNNEST { array_exprs, .. } => {
            walk_table_exprs(sql, array_exprs, ctes, selects);
        }
        TableFactor::Table {
            args: Some(args), ..
        } => {
            crate::codebase::postgres::idents::visit_function_args(&args.args, &mut |expr| {
                record_table_expr(sql, expr, ctes, selects);
            });
        }
        _ => {}
    }
}

fn walk_table_exprs(sql: &str, exprs: &[Expr], ctes: &[String], selects: &mut Vec<SqlSelectFact>) {
    for expr in exprs {
        record_table_expr(sql, expr, ctes, selects);
    }
}

fn record_table_expr(sql: &str, expr: &Expr, ctes: &[String], selects: &mut Vec<SqlSelectFact>) {
    super::super::select::collect_predicate_shapes(expr, selects);
    collect_exists_facts(sql, expr, selects);
    super::super::select::walk_expr(sql, expr, ctes, false, selects);
}

/// Record `EXISTS (... UNION ...)` wrappers that sit outside a SELECT.
pub(super) fn collect_exists_facts(sql: &str, expr: &Expr, selects: &mut Vec<SqlSelectFact>) {
    let mut exists_set_operations = Vec::new();
    super::super::exists::collect_exists(sql, Some(expr), &mut exists_set_operations);
    if exists_set_operations.is_empty() {
        return;
    }
    selects.push(SqlSelectFact {
        line: 1,
        tables: Vec::new(),
        predicate_sql: String::new(),
        exists_set_operations,
        relations: Vec::new(),
        in_insert_select: false,
        not_in_subqueries: Vec::new(),
        not_in_columns: Vec::new(),
        count_existence_checks: Vec::new(),
        star_projections: Vec::new(),
        column_uses: Vec::new(),
    });
}
