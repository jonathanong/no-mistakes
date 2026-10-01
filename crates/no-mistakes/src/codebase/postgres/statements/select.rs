mod from;
mod nested;
mod shapes;

pub(super) fn join_expr(operator: &sqlparser::ast::JoinOperator) -> Option<&sqlparser::ast::Expr> {
    from::join_expr(operator)
}

pub(super) fn walk_expr(
    sql: &str,
    expr: &sqlparser::ast::Expr,
    ctes: &[String],
    in_insert_select: bool,
    out: &mut Vec<SqlSelectFact>,
) {
    nested::walk_expr(sql, expr, ctes, in_insert_select, out);
}

use super::SqlSelectFact;
use sqlparser::ast::{Query, Select, SetExpr, Statement};

pub(super) fn collect(sql: &str, statement: &Statement, out: &mut Vec<SqlSelectFact>) {
    match statement {
        Statement::Query(query) => collect_query(sql, query, &[], false, out),
        Statement::Insert(insert) => {
            if let Some(source) = insert.source.as_deref() {
                collect_query(sql, source, &[], true, out);
            }
        }
        Statement::CreateView(view) => collect_query(sql, &view.query, &[], false, out),
        Statement::CreateTable(table) => {
            if let Some(query) = table.query.as_deref() {
                collect_query(sql, query, &[], false, out);
            }
        }
        Statement::Copy {
            source: sqlparser::ast::CopySource::Query(query),
            ..
        } => collect_query(sql, query, &[], false, out),
        Statement::Explain { statement, .. } => collect(sql, statement, out),
        _ => {}
    }
}

pub(super) fn collect_query(
    sql: &str,
    query: &Query,
    outer_ctes: &[String],
    in_insert_select: bool,
    out: &mut Vec<SqlSelectFact>,
) {
    let mut ctes = outer_ctes.to_vec();
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            collect_query(sql, &cte.query, &ctes, in_insert_select, out);
            ctes.push(cte.alias.name.value.clone());
        }
    }
    collect_set(sql, &query.body, &ctes, in_insert_select, out);
}

fn collect_set(
    sql: &str,
    expr: &SetExpr,
    ctes: &[String],
    in_insert_select: bool,
    out: &mut Vec<SqlSelectFact>,
) {
    match expr {
        SetExpr::Select(select) => push_select(sql, select, ctes, in_insert_select, out),
        SetExpr::Query(query) => collect_query(sql, query, ctes, in_insert_select, out),
        SetExpr::SetOperation { left, right, .. } => {
            collect_set(sql, left, ctes, in_insert_select, out);
            collect_set(sql, right, ctes, in_insert_select, out);
        }
        _ => {}
    }
}

fn push_select(
    sql: &str,
    select: &Select,
    ctes: &[String],
    in_insert_select: bool,
    out: &mut Vec<SqlSelectFact>,
) {
    let tables = from::table_names(&select.from, ctes);
    let mut exists_set_operations = Vec::new();
    super::exists::collect_from_select(sql, select, &mut exists_set_operations);
    from::collect_derived_queries(sql, &select.from, ctes, in_insert_select, out);
    nested::collect(sql, select, ctes, in_insert_select, out);
    let relations = super::predicates::select_relations(sql, select, ctes);
    let shapes = shapes::collect(sql, select);
    if tables.is_empty()
        && exists_set_operations.is_empty()
        && relations.is_empty()
        && shapes.not_in_subqueries.is_empty()
        && shapes.count_existence_checks.is_empty()
    {
        return;
    }
    out.push(SqlSelectFact {
        line: super::lines::line_containing(
            sql,
            &[tables.first().map(String::as_str).unwrap_or("select")],
        ),
        tables,
        predicate_sql: from::predicate_text(select),
        exists_set_operations,
        relations,
        in_insert_select,
        not_in_subqueries: shapes.not_in_subqueries,
        count_existence_checks: shapes.count_existence_checks,
    });
}

#[cfg(test)]
mod tests;
