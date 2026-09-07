//! Typed INSERT, SELECT, and trigger facts from PostgreSQL SQL.

mod conflict;
mod exists;
mod fallback;
mod insert;
mod lines;
mod not_exists;
mod select;
mod trigger;
mod value;
mod wrappers;

pub use crate::codebase::postgres::statement_facts::*;
pub use fallback::{insert_keyword_count, mask_quoted_sql};

use crate::codebase::postgres::parse::{parse_postgres_sql, parse_postgres_sql_lenient};
use sqlparser::ast::{Query, SetExpr, Statement};

/// Extract INSERT/SELECT/trigger facts from one SQL source.
pub fn extract_sql_statement_facts(sql: &str) -> SqlStatementFileFacts {
    let masked = mask_sql(sql);
    let insert_keyword_count = fallback::insert_keyword_count(&masked);
    let parse_failed = parse_postgres_sql(sql).is_err();
    let statements = parse_postgres_sql_lenient(sql);
    let mut inserts = Vec::new();
    let mut selects = Vec::new();
    let mut triggers = Vec::new();
    let mut insert_n = 0usize;
    let mut trigger_n = 0usize;
    let mut executed = Vec::new();
    for statement in &statements {
        wrappers::walk_executed(statement, &mut executed);
    }
    for statement in executed {
        collect_one(
            sql,
            statement,
            &mut insert_n,
            &mut trigger_n,
            &mut inserts,
            &mut selects,
            &mut triggers,
        );
    }
    SqlStatementFileFacts {
        path: Default::default(),
        inserts,
        selects,
        triggers,
        parse_failed,
        insert_keyword_count,
        has_top_level_not_exists: not_exists::has_top_level_conjunctive_not_exists(&masked),
        origin_line: 0,
    }
}

fn collect_one(
    sql: &str,
    statement: &Statement,
    insert_n: &mut usize,
    trigger_n: &mut usize,
    inserts: &mut Vec<SqlInsertFact>,
    selects: &mut Vec<SqlSelectFact>,
    triggers: &mut Vec<SqlTriggerFact>,
) {
    if let Statement::Insert(insert) = statement {
        *insert_n += 1;
        if let Some(fact) = insert::from_statement(sql, statement, *insert_n) {
            inserts.push(fact);
        }
        if let Some(source) = insert.source.as_deref() {
            collect_query_inserts(sql, source, insert_n, inserts);
        }
    }
    if matches!(statement, Statement::CreateTrigger(_)) {
        *trigger_n += 1;
        if let Some(fact) = trigger::from_statement(sql, statement, *trigger_n) {
            triggers.push(fact);
        }
    }
    if let Statement::Query(query) = statement {
        collect_query_inserts(sql, query, insert_n, inserts);
    }
    select::collect(sql, statement, selects);
}

fn collect_query_inserts(
    sql: &str,
    query: &Query,
    insert_n: &mut usize,
    inserts: &mut Vec<SqlInsertFact>,
) {
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            collect_query_inserts(sql, &cte.query, insert_n, inserts);
        }
    }
    collect_set_inserts(sql, &query.body, insert_n, inserts);
}

fn collect_set_inserts(
    sql: &str,
    expr: &SetExpr,
    insert_n: &mut usize,
    inserts: &mut Vec<SqlInsertFact>,
) {
    match expr {
        SetExpr::Insert(statement) => {
            if matches!(statement, Statement::Insert(_)) {
                *insert_n += 1;
                if let Some(fact) = insert::from_statement(sql, statement, *insert_n) {
                    inserts.push(fact);
                }
            }
        }
        SetExpr::Query(query) => collect_query_inserts(sql, query, insert_n, inserts),
        SetExpr::SetOperation { left, right, .. } => {
            collect_set_inserts(sql, left, insert_n, inserts);
            collect_set_inserts(sql, right, insert_n, inserts);
        }
        _ => {}
    }
}

pub fn has_top_level_not_exists_in(sql: &str) -> bool {
    not_exists::has_top_level_conjunctive_not_exists(&mask_sql(sql))
}

fn mask_sql(sql: &str) -> String {
    fallback::mask_quoted_sql(&fallback::mask_comments(sql))
}

#[cfg(test)]
mod tests;
