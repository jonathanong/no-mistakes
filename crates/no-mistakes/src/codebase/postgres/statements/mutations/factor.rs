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
    super::super::select::walk_expr(sql, expr, ctes, false, selects);
}
