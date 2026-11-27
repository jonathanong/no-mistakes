use crate::SqlWriteFact;
use sqlparser::ast::{Query, SetExpr, Statement};

pub(super) fn walk(statement: &Statement, out: &mut Vec<SqlWriteFact>) {
    match statement {
        Statement::Query(query) => walk_query(query, out),
        Statement::Insert(insert) => {
            if let Some(source) = &insert.source {
                walk_query(source, out);
            }
        }
        _ => {}
    }
}

fn walk_query(query: &Query, out: &mut Vec<SqlWriteFact>) {
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            walk_query(&cte.query, out);
        }
    }
    walk_set(&query.body, out);
}

fn walk_set(expr: &SetExpr, out: &mut Vec<SqlWriteFact>) {
    match expr {
        SetExpr::Insert(statement)
        | SetExpr::Update(statement)
        | SetExpr::Delete(statement)
        | SetExpr::Merge(statement) => super::collect(statement, out),
        SetExpr::Query(query) => walk_query(query, out),
        SetExpr::SetOperation { left, right, .. } => {
            walk_set(left, out);
            walk_set(right, out);
        }
        _ => {}
    }
}
