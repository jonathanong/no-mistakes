//! Row-count bound facts for executed `SELECT`, `UPDATE` and `DELETE` statements.
//!
//! The facts are syntactic. Each FROM item keeps the columns the statement equates with a
//! value, with another item, or with a subquery, and each query keeps whether a `LIMIT` or a
//! pure aggregate caps it. A rule decides, against a schema catalog, whether those pins make
//! a relation single-row.
mod aggregate;
mod compact_pins;
mod dml;
mod functions;
mod items;
mod pins;
mod query;
mod table;
mod temporary;
use table::TableTokenCursor;
pub(super) use table::TableTokenIndex;
pub(super) use temporary::TemporaryRelations;
#[cfg(test)]
mod tests;
mod using;

use super::{SqlBoundFact, SqlBoundKind, SqlBoundQuery};
use sqlparser::ast::{CopySource, Query, Spanned, Statement};
use std::collections::{BTreeMap, BTreeSet};
use std::{cell::RefCell, rc::Rc};

/// CTE names in scope, with the bound of each.
#[derive(Clone, Default)]
pub(super) struct Scope {
    ctes: BTreeMap<String, SqlBoundQuery>,
    columns: BTreeMap<String, Option<BTreeSet<String>>>,
    table_tokens: Option<Rc<RefCell<TableTokenCursor>>>,
}

impl Scope {
    pub(super) fn with_table_tokens(tokens: TableTokenCursor) -> Self {
        Self {
            ctes: BTreeMap::new(),
            columns: BTreeMap::new(),
            table_tokens: Some(Rc::new(RefCell::new(tokens))),
        }
    }

    fn get(&self, name: &str) -> Option<&SqlBoundQuery> {
        self.ctes.get(name)
    }

    fn names(&self) -> BTreeMap<String, Option<BTreeSet<String>>> {
        self.columns.clone()
    }

    fn insert(&mut self, name: String, bound: SqlBoundQuery) {
        self.columns.entry(name.clone()).or_insert(None);
        self.ctes.insert(name, bound);
    }
}

pub(super) fn collect(statement: &Statement, scope: &Scope, out: &mut Vec<SqlBoundFact>) {
    collect_in(statement, scope, out);
}

fn collect_in(statement: &Statement, scope: &Scope, out: &mut Vec<SqlBoundFact>) {
    match statement {
        Statement::Query(query) => collect_query(query, scope, out),
        // `COPY (SELECT …) TO` runs the query like any other.
        Statement::Copy {
            source: CopySource::Query(query),
            ..
        } => collect_query(query, scope, out),
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
    let (line, column) = start(query.span());
    out.push(SqlBoundFact {
        kind: SqlBoundKind::Select,
        line,
        column,
        query: query::bound_body(query, &scope),
        target: None,
    });
}

/// The 1-based line and column where `span` starts.
fn start(span: sqlparser::tokenizer::Span) -> (usize, usize) {
    (
        (span.start.line as usize).max(1),
        (span.start.column as usize).max(1),
    )
}
