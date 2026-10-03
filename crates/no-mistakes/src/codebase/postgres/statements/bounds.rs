//! Row-count bound facts for executed `SELECT`, `UPDATE` and `DELETE` statements.
//!
//! The facts are syntactic. Each FROM item keeps the columns the statement equates with a
//! value, with another item, or with a subquery, and each query keeps whether a `LIMIT` or a
//! pure aggregate caps it. A rule decides, against a schema catalog, whether those pins make
//! a relation single-row.
mod dml;
mod items;
mod pins;
mod query;
#[cfg(test)]
mod tests;

use super::{SqlBoundFact, SqlBoundKind, SqlBoundQuery};
use sqlparser::ast::{Query, Spanned, Statement};
use std::collections::BTreeMap;

/// CTE names in scope, with the bound of each.
#[derive(Clone, Default)]
pub(super) struct Scope {
    ctes: BTreeMap<String, SqlBoundQuery>,
}

impl Scope {
    fn get(&self, name: &str) -> Option<&SqlBoundQuery> {
        self.ctes.get(name)
    }

    fn insert(&mut self, name: String, bound: SqlBoundQuery) {
        self.ctes.insert(name, bound);
    }
}

pub(super) fn collect(statement: &Statement, out: &mut Vec<SqlBoundFact>) {
    collect_in(statement, &Scope::default(), out);
}

fn collect_in(statement: &Statement, scope: &Scope, out: &mut Vec<SqlBoundFact>) {
    match statement {
        Statement::Query(query) => collect_query(query, scope, out),
        Statement::Update(update) => dml::update(update, scope, out),
        Statement::Delete(delete) => dml::delete(delete, scope, out),
        _ => {}
    }
}

fn collect_query(query: &Query, scope: &Scope, out: &mut Vec<SqlBoundFact>) {
    let scope = query::with_scope(query, scope);
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            if let Some(statement) = query::modifying_statement(&cte.query) {
                collect_in(statement, &scope, out);
            }
        }
    }
    // `WITH … UPDATE` parses as a query whose body is the statement.
    if let Some(statement) = query::modifying_statement(query) {
        collect_in(statement, &scope, out);
        return;
    }
    out.push(SqlBoundFact {
        kind: SqlBoundKind::Select,
        line: line(query.span()),
        query: query::bound_body(query, &scope),
        target: None,
    });
}

fn line(span: sqlparser::tokenizer::Span) -> usize {
    (span.start.line as usize).max(1)
}
