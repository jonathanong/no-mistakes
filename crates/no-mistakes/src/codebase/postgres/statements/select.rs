mod nested;

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
use sqlparser::ast::{
    Expr, JoinConstraint, Query, Select, SetExpr, Statement, TableFactor, TableWithJoins,
};

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
    let tables = table_names(&select.from, ctes);
    let mut exists_set_operations = Vec::new();
    super::exists::collect_from_select(sql, select, &mut exists_set_operations);
    collect_derived_queries(sql, &select.from, ctes, in_insert_select, out);
    nested::collect(sql, select, ctes, in_insert_select, out);
    let relations = super::predicates::select_relations(sql, select, ctes);
    if tables.is_empty() && exists_set_operations.is_empty() && relations.is_empty() {
        return;
    }
    out.push(SqlSelectFact {
        line: super::lines::line_containing(
            sql,
            &[tables.first().map(String::as_str).unwrap_or("select")],
        ),
        tables,
        predicate_sql: predicate_text(select),
        exists_set_operations,
        relations,
        in_insert_select,
    });
}

fn table_names(from: &[TableWithJoins], ctes: &[String]) -> Vec<String> {
    let mut names = Vec::new();
    for table in from {
        push_table(&table.relation, ctes, &mut names);
        for join in &table.joins {
            push_table(&join.relation, ctes, &mut names);
        }
    }
    names
}

fn collect_derived_queries(
    sql: &str,
    from: &[TableWithJoins],
    ctes: &[String],
    in_insert_select: bool,
    out: &mut Vec<SqlSelectFact>,
) {
    for table in from {
        collect_derived_factor(sql, &table.relation, ctes, in_insert_select, out);
        for join in &table.joins {
            collect_derived_factor(sql, &join.relation, ctes, in_insert_select, out);
        }
    }
}

fn collect_derived_factor(
    sql: &str,
    table: &TableFactor,
    ctes: &[String],
    in_insert_select: bool,
    out: &mut Vec<SqlSelectFact>,
) {
    match table {
        TableFactor::Derived { subquery, .. } => {
            collect_query(sql, subquery, ctes, in_insert_select, out);
        }
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => collect_derived_queries(
            sql,
            std::slice::from_ref(table_with_joins),
            ctes,
            in_insert_select,
            out,
        ),
        _ => {}
    }
}

fn push_table(table: &TableFactor, ctes: &[String], names: &mut Vec<String>) {
    match table {
        TableFactor::Table { name, .. } => {
            if let Some(table) = super::predicates::base_table(name, ctes) {
                names.push(table);
            }
        }
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => names.extend(table_names(std::slice::from_ref(table_with_joins), ctes)),
        _ => {}
    }
}

fn predicate_text(select: &Select) -> String {
    let mut parts = Vec::new();
    if let Some(selection) = &select.selection {
        parts.push(selection.to_string());
    }
    for table in &select.from {
        for join in &table.joins {
            if let Some(expr) = join_expr(&join.join_operator) {
                parts.push(expr.to_string());
            }
        }
    }
    parts.join(" ")
}

pub(super) fn join_expr(operator: &sqlparser::ast::JoinOperator) -> Option<&Expr> {
    match operator {
        sqlparser::ast::JoinOperator::Join(JoinConstraint::On(expr))
        | sqlparser::ast::JoinOperator::Inner(JoinConstraint::On(expr))
        | sqlparser::ast::JoinOperator::Left(JoinConstraint::On(expr))
        | sqlparser::ast::JoinOperator::LeftOuter(JoinConstraint::On(expr))
        | sqlparser::ast::JoinOperator::Right(JoinConstraint::On(expr))
        | sqlparser::ast::JoinOperator::RightOuter(JoinConstraint::On(expr))
        | sqlparser::ast::JoinOperator::FullOuter(JoinConstraint::On(expr)) => Some(expr),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
