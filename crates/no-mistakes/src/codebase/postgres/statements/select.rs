use super::SqlSelectFact;
use crate::codebase::postgres::schema::relation_name;
use sqlparser::ast::{
    Expr, JoinConstraint, Query, Select, SetExpr, Statement, TableFactor, TableWithJoins,
};

pub(super) fn collect(sql: &str, statement: &Statement, out: &mut Vec<SqlSelectFact>) {
    match statement {
        Statement::Query(query) => collect_query(sql, query, out),
        Statement::Insert(insert) => {
            if let Some(source) = insert.source.as_deref() {
                collect_query(sql, source, out);
            }
        }
        Statement::CreateView(view) => collect_query(sql, &view.query, out),
        Statement::Explain { statement, .. } => collect(sql, statement, out),
        _ => {}
    }
}

fn collect_query(sql: &str, query: &Query, out: &mut Vec<SqlSelectFact>) {
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            collect_query(sql, &cte.query, out);
        }
    }
    collect_set(sql, &query.body, out);
}

fn collect_set(sql: &str, expr: &SetExpr, out: &mut Vec<SqlSelectFact>) {
    match expr {
        SetExpr::Select(select) => push_select(sql, select, out),
        SetExpr::Query(query) => collect_query(sql, query, out),
        SetExpr::SetOperation { left, right, .. } => {
            collect_set(sql, left, out);
            collect_set(sql, right, out);
        }
        _ => {}
    }
}

fn push_select(sql: &str, select: &Select, out: &mut Vec<SqlSelectFact>) {
    let tables = table_names(&select.from);
    let mut exists_set_operations = Vec::new();
    super::exists::collect_exists(select.selection.as_ref(), &mut exists_set_operations);
    for table in &select.from {
        for join in &table.joins {
            if let Some(expr) = join_expr(&join.join_operator) {
                super::exists::collect_exists(Some(expr), &mut exists_set_operations);
            }
        }
    }
    collect_derived_queries(sql, &select.from, out);
    if tables.is_empty() && exists_set_operations.is_empty() {
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
    });
}

fn table_names(from: &[TableWithJoins]) -> Vec<String> {
    let mut names = Vec::new();
    for table in from {
        push_table(&table.relation, &mut names);
        for join in &table.joins {
            push_table(&join.relation, &mut names);
        }
    }
    names
}

fn collect_derived_queries(sql: &str, from: &[TableWithJoins], out: &mut Vec<SqlSelectFact>) {
    for table in from {
        collect_derived_factor(sql, &table.relation, out);
        for join in &table.joins {
            collect_derived_factor(sql, &join.relation, out);
        }
    }
}

fn collect_derived_factor(sql: &str, table: &TableFactor, out: &mut Vec<SqlSelectFact>) {
    match table {
        TableFactor::Derived { subquery, .. } => collect_query(sql, subquery, out),
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => collect_derived_queries(sql, std::slice::from_ref(table_with_joins), out),
        _ => {}
    }
}

fn push_table(table: &TableFactor, names: &mut Vec<String>) {
    match table {
        TableFactor::Table { name, .. } => names.push(relation_name(name)),
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => names.extend(table_names(std::slice::from_ref(table_with_joins))),
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

fn join_expr(operator: &sqlparser::ast::JoinOperator) -> Option<&Expr> {
    match operator {
        sqlparser::ast::JoinOperator::Inner(JoinConstraint::On(expr))
        | sqlparser::ast::JoinOperator::LeftOuter(JoinConstraint::On(expr))
        | sqlparser::ast::JoinOperator::RightOuter(JoinConstraint::On(expr))
        | sqlparser::ast::JoinOperator::FullOuter(JoinConstraint::On(expr)) => Some(expr),
        _ => None,
    }
}
