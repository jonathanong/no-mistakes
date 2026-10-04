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
mod predicate;
mod query;
mod table;
mod temporary;
pub(in crate::codebase::postgres) use table::TableTokenCursor;
pub(in crate::codebase::postgres) use table::TableTokenIndex;
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
    frame: Rc<ScopeFrame>,
    table_tokens: Option<Rc<RefCell<TableTokenCursor>>>,
}

/// Immutable parent frames retain visible CTEs without copying their bound trees.
#[derive(Clone, Default)]
struct ScopeFrame {
    parent: Option<Rc<ScopeFrame>>,
    ctes: BTreeMap<String, Rc<SqlBoundQuery>>,
    columns: BTreeMap<String, Option<BTreeSet<String>>>,
}

impl Scope {
    pub(super) fn with_table_tokens(tokens: TableTokenCursor) -> Self {
        Self {
            frame: Rc::default(),
            table_tokens: Some(Rc::new(RefCell::new(tokens))),
        }
    }

    fn advance_table_tokens_to_right_arm(&self, left_start: sqlparser::tokenizer::Location) {
        if let Some(tokens) = &self.table_tokens {
            tokens.borrow_mut().advance_to_right_arm(left_start);
        }
    }

    fn advance_table_tokens_to_from(&self, select_start: sqlparser::tokenizer::Location) {
        if let Some(tokens) = &self.table_tokens {
            tokens.borrow_mut().advance_to_from(select_start);
        }
    }

    fn get(&self, name: &str) -> Option<&SqlBoundQuery> {
        let mut frame = self.frame.as_ref();
        loop {
            if let Some(bound) = frame.ctes.get(name) {
                return Some(bound.as_ref());
            }
            frame = frame.parent.as_deref()?;
        }
    }

    fn names(&self) -> BTreeMap<String, Option<BTreeSet<String>>> {
        let mut frames = Vec::new();
        let mut frame = Some(self.frame.as_ref());
        while let Some(current) = frame {
            frames.push(current);
            frame = current.parent.as_deref();
        }
        let mut names = BTreeMap::new();
        for frame in frames.into_iter().rev() {
            names.extend(frame.columns.clone());
        }
        names
    }

    fn child(&self) -> Self {
        Self {
            frame: Rc::new(ScopeFrame {
                parent: Some(Rc::clone(&self.frame)),
                ..ScopeFrame::default()
            }),
            table_tokens: self.table_tokens.clone(),
        }
    }

    fn set_columns(&mut self, name: String, columns: Option<BTreeSet<String>>) {
        Rc::make_mut(&mut self.frame).columns.insert(name, columns);
    }

    fn column_names(&self, name: &str) -> Option<&Option<BTreeSet<String>>> {
        let mut frame = self.frame.as_ref();
        loop {
            if let Some(columns) = frame.columns.get(name) {
                return Some(columns);
            }
            frame = frame.parent.as_deref()?;
        }
    }

    fn insert(&mut self, name: String, bound: SqlBoundQuery) {
        // Opaque recursive placeholders retain the metadata visible before replacement.
        let columns = self.column_names(&name).cloned().unwrap_or_default();
        let frame = Rc::make_mut(&mut self.frame);
        frame.columns.entry(name.clone()).or_insert(columns);
        frame.ctes.insert(name, Rc::new(bound));
    }
}

pub(super) fn collect(
    statement: &Statement,
    scope: &Scope,
    positions: super::value::PlaceholderPositions<'_>,
    out: &mut Vec<SqlBoundFact>,
) {
    collect_in(statement, scope, positions, out);
}

fn collect_in(
    statement: &Statement,
    scope: &Scope,
    positions: super::value::PlaceholderPositions<'_>,
    out: &mut Vec<SqlBoundFact>,
) {
    match statement {
        Statement::Query(query) => collect_query(query, scope, positions, out),
        // `COPY (SELECT …) TO` runs the query like any other.
        Statement::Copy {
            source: CopySource::Query(query),
            ..
        } => collect_query(query, scope, positions, out),
        Statement::Update(update) => dml::update(update, scope, positions, out),
        Statement::Delete(delete) => dml::delete(delete, scope, positions, out),
        _ => {}
    }
}

fn collect_query(
    query: &Query,
    scope: &Scope,
    positions: super::value::PlaceholderPositions<'_>,
    out: &mut Vec<SqlBoundFact>,
) {
    let scope = query::with_scope(query, scope, positions);
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            if let Some(statement) = query::modifying_statement(&cte.query) {
                collect_in(statement, &scope, positions, out);
            }
        }
    }
    // `WITH … UPDATE` parses as a query whose body is the statement.
    if let Some(statement) = query::modifying_statement(query) {
        collect_in(statement, &scope, positions, out);
        return;
    }
    let (line, column) = start(query.span());
    out.push(SqlBoundFact {
        kind: SqlBoundKind::Select,
        line,
        column,
        statement_start: None,
        query: query::bound_body(query, &scope, positions),
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
