//! Typed INSERT, SELECT, and trigger facts from PostgreSQL SQL.

mod bounds;
mod conflict;
mod dedupe;
mod exists;
mod exists_correlation;
mod fallback;
mod insert;
mod insert_source;
mod limit;
mod lines;
mod mutations;
mod not_exists;
mod predicates;
mod select;
mod trigger;
mod value;
mod wrappers;
mod writes;

pub use crate::codebase::postgres::statement_facts::*;
pub use fallback::{insert_keyword_count, mask_quoted_sql};
pub(crate) use value::form_is_stable;
pub(crate) use wrappers::walk_executed;

use crate::codebase::postgres::parse::{parse_postgres_sql, parse_postgres_sql_lenient};
use sqlparser::ast::{Query, SetExpr, Statement};

/// Extract INSERT/SELECT/trigger facts from one SQL source.
pub fn extract_sql_statement_facts(sql: &str) -> SqlStatementFileFacts {
    extract_sql_statement_facts_with_bounds(sql, true)
}

/// Extract the requested SQL projections without reparsing for row bounds.
pub(crate) fn extract_sql_statement_facts_with_bounds(
    sql: &str,
    collect_bounds: bool,
) -> SqlStatementFileFacts {
    let parsed = parse_postgres_sql(sql);
    let parse_failed = parsed.is_err();
    let statements = parsed.unwrap_or_else(|_| parse_postgres_sql_lenient(sql));
    extract_from_parsed(sql, &statements, parse_failed, collect_bounds)
}

pub(crate) fn extract_from_parsed(
    sql: &str,
    statements: &[Statement],
    parse_failed: bool,
    collect_bounds: bool,
) -> SqlStatementFileFacts {
    let masked = fallback::mask_quoted_sql(sql);
    let insert_keyword_count = fallback::insert_keyword_count(&masked);
    let mut writes = Vec::new();
    let mut inserts = Vec::new();
    let mut selects = Vec::new();
    let mut updates = Vec::new();
    let mut deletes = Vec::new();
    let mut triggers = Vec::new();
    let mut returning_stars = Vec::new();
    let mut bounds = Vec::new();
    let mut mutation_column_uses = Vec::new();
    let mut insert_n = 0usize;
    let mut trigger_n = 0usize;
    let mut executed = Vec::new();
    for statement in statements {
        wrappers::walk_executed(statement, &mut executed);
    }
    let mut out = FactOut {
        insert_n: &mut insert_n,
        trigger_n: &mut trigger_n,
        inserts: &mut inserts,
        selects: &mut selects,
        updates: &mut updates,
        deletes: &mut deletes,
        triggers: &mut triggers,
        returning_stars: &mut returning_stars,
        mutation_column_uses: &mut mutation_column_uses,
    };
    for statement in executed {
        writes::collect(statement, &mut writes);
        collect_one(sql, statement, &mut out);
        if collect_bounds {
            bounds::collect(statement, &mut bounds);
        }
    }
    dedupe::exists_set_operations(&mut selects);
    SqlStatementFileFacts {
        path: Default::default(),
        writes,
        inserts,
        selects,
        updates,
        deletes,
        triggers,
        returning_stars,
        mutation_column_uses,
        offset_uses: super::offset::offset_facts(sql, statements),
        bounds,
        parse_failed,
        insert_keyword_count,
        has_top_level_not_exists: not_exists::has_top_level_conjunctive_not_exists(&masked),
        origin_line: 0,
    }
}

struct FactOut<'a> {
    insert_n: &'a mut usize,
    trigger_n: &'a mut usize,
    inserts: &'a mut Vec<SqlInsertFact>,
    selects: &'a mut Vec<SqlSelectFact>,
    updates: &'a mut Vec<Vec<SqlRelationPredicateFact>>,
    deletes: &'a mut Vec<Vec<SqlRelationPredicateFact>>,
    triggers: &'a mut Vec<SqlTriggerFact>,
    returning_stars: &'a mut Vec<SqlStarProjectionFact>,
    mutation_column_uses: &'a mut Vec<SqlColumnUseFact>,
}

fn collect_one(sql: &str, statement: &Statement, out: &mut FactOut<'_>) {
    if let Statement::Insert(insert) = statement {
        *out.insert_n += 1;
        if let Some(fact) = insert::from_statement(sql, statement, *out.insert_n) {
            out.inserts.push(fact);
        }
        if let Some(source) = insert.source.as_deref() {
            collect_query_inserts(sql, source, out.insert_n, out.inserts);
        }
    }
    if matches!(statement, Statement::CreateTrigger(_)) {
        *out.trigger_n += 1;
        if let Some(fact) = trigger::from_statement(sql, statement, *out.trigger_n) {
            out.triggers.push(fact);
        }
    }
    if let Statement::Query(query) = statement {
        collect_query_inserts(sql, query, out.insert_n, out.inserts);
    }
    select::collect(sql, statement, out.selects);
    out.returning_stars
        .extend(select::returning_stars(sql, statement));
    mutations::collect(
        sql,
        statement,
        out.updates,
        out.deletes,
        out.selects,
        out.mutation_column_uses,
    );
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
    not_exists::has_top_level_conjunctive_not_exists(&fallback::mask_quoted_sql(sql))
}

#[cfg(test)]
mod ast_coverage_tests;
#[cfg(test)]
mod builtin_form_tests;
#[cfg(test)]
mod coverage_mask_tests;
#[cfg(test)]
mod coverage_more_tests;
#[cfg(test)]
mod coverage_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod value_ast_tests;
