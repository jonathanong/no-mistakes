//! Accepted unique-key domain products, including nested statements.
use super::keys::Credit;
use super::Evaluation;
use crate::codebase::postgres::statements::{SqlBoundItemKind, SqlBoundQuery};
use crate::codebase::postgres::SchemaCatalog;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::codebase::rules::postgres_bounded_statements) struct DomainAccept {
    pub(in crate::codebase::rules::postgres_bounded_statements) table: String,
    pub(in crate::codebase::rules::postgres_bounded_statements) line: usize,
    pub(in crate::codebase::rules::postgres_bounded_statements) product: u64,
    pub(in crate::codebase::rules::postgres_bounded_statements) cap: u64,
}

pub(super) fn collect(
    query: &SqlBoundQuery,
    bounded: &[bool],
    credits: &[Option<Credit>],
    nested: &[Option<Evaluation>],
    pin_subqueries: &[Vec<Option<Evaluation>>],
    catalog: &SchemaCatalog,
    cap: Option<u64>,
) -> Vec<DomainAccept> {
    let mut accepts = Vec::new();
    for evaluation in nested.iter().flatten() {
        accepts.extend(evaluation.accepts.iter().cloned());
    }
    for queries in pin_subqueries {
        for evaluation in queries.iter().flatten() {
            accepts.extend(evaluation.accepts.iter().cloned());
        }
    }
    let Some(cap) = cap else {
        return accepts;
    };
    for (index, item) in query.items.iter().enumerate() {
        let SqlBoundItemKind::Table(name) = &item.kind else {
            continue;
        };
        let Some(table) = catalog.bounded_relation(name) else {
            continue;
        };
        if !bounded.get(index).copied().unwrap_or(false) {
            continue;
        }
        let Some(Credit::Accept(product)) = credits.get(index).copied().flatten() else {
            continue;
        };
        accepts.push(DomainAccept {
            table: table.name.clone(),
            line: item.line,
            product,
            cap,
        });
    }
    accepts
}
