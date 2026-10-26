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
mod scalar_limit;
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

/// Share the canonical builtin aggregate identity with other statement projections.
pub(super) fn is_aggregate(name: &sqlparser::ast::ObjectName) -> bool {
    functions::is_aggregate(name)
}

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
    fixed_counts: BTreeMap<String, BTreeSet<String>>,
}

mod scope;

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
    let scope = query::with_scope_visiting_writes(query, scope, positions, |cte, visible| {
        collect_query(cte, visible, positions, out);
    });
    // RECURSIVE exposes every sibling; keep its completed scope and declaration-order facts.
    if let Some(with) = query.with.as_ref().filter(|with| with.recursive) {
        for cte in &with.cte_tables {
            if query::modifying_statement(&cte.query).is_some() {
                // A modifying CTE can itself have WITH bindings visible to its target query.
                collect_query(&cte.query, &scope, positions, out);
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
