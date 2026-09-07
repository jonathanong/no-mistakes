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
use sqlparser::ast::Statement;

/// Extract INSERT/SELECT/trigger facts from one SQL source.
pub fn extract_sql_statement_facts(sql: &str) -> SqlStatementFileFacts {
    let masked = fallback::mask_comments(&fallback::mask_quoted_sql(sql));
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
    if matches!(statement, Statement::Insert(_)) {
        *insert_n += 1;
        if let Some(fact) = insert::from_statement(sql, statement, *insert_n) {
            inserts.push(fact);
        }
    }
    if matches!(statement, Statement::CreateTrigger(_)) {
        *trigger_n += 1;
        if let Some(fact) = trigger::from_statement(sql, statement, *trigger_n) {
            triggers.push(fact);
        }
    }
    select::collect(sql, statement, selects);
}

pub fn has_top_level_not_exists_in(sql: &str) -> bool {
    not_exists::has_top_level_conjunctive_not_exists(&fallback::mask_comments(
        &fallback::mask_quoted_sql(sql),
    ))
}

#[cfg(test)]
mod tests;
