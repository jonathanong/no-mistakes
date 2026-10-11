//! Decide, against a schema catalog, whether a statement's rows are bounded.
use crate::codebase::postgres::statements::{
    SqlBareRead, SqlBoundFact, SqlBoundInputMode, SqlBoundItemKind, SqlBoundQuery, SqlPinSource,
};
use crate::codebase::postgres::SchemaCatalog;

mod possible_temporary;
mod reads;
use reads::table_offender;
mod keys;
mod propagation;
mod qualified;

/// A catalog relation that a statement can read or change in unbounded numbers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Offender {
    /// The catalog's name for the table.
    pub(super) table: String,
    pub(super) line: usize,
    pub(super) column: usize,
    blocking_input: bool,
}

struct Evaluation {
    /// A `LIMIT`, a pure aggregate, or every FROM item bounded.
    bounded: bool,
    /// Per FROM item: whether that item is bounded.
    items: Vec<bool>,
    offenders: Vec<Offender>,
}

/// The relations that make `fact` unbounded. A relation the catalog does not know is not
/// judged, and it bounds nothing either: it can supply every value of a column pinned to it.
pub(super) fn offenders(fact: &SqlBoundFact, catalog: &SchemaCatalog) -> Vec<Offender> {
    offenders_with_columns(fact, catalog, false)
}

pub(super) fn variant_offenders(fact: &SqlBoundFact, catalog: &SchemaCatalog) -> Vec<Offender> {
    offenders_with_columns(fact, catalog, true)
}

fn offenders_with_columns(
    fact: &SqlBoundFact,
    catalog: &SchemaCatalog,
    precise_columns: bool,
) -> Vec<Offender> {
    let query = possible_temporary::project(&fact.query, catalog);
    let evaluation = evaluate(&query, catalog, precise_columns);
    let Some(target) = fact.target else {
        return evaluation.offenders;
    };
    // An UPDATE or DELETE changes each target row at most once, so only the target must be bounded.
    if query.capped || evaluation.items[target] {
        return Vec::new();
    }
    let item = &query.items[target];
    match &item.kind {
        SqlBoundItemKind::Table(name) => table_offender(name, item.line, item.column, catalog)
            .into_iter()
            .collect(),
        _ => Vec::new(),
    }
}

fn evaluate(query: &SqlBoundQuery, catalog: &SchemaCatalog, precise_columns: bool) -> Evaluation {
    if query.input_mode == SqlBoundInputMode::Skipped {
        return Evaluation {
            bounded: true,
            items: vec![true; query.items.len()],
            offenders: Vec::new(),
        };
    }
    let nested: Vec<Option<Evaluation>> = query
        .items
        .iter()
        .map(|item| match &item.kind {
            SqlBoundItemKind::Query(inner) => Some(evaluate(inner, catalog, precise_columns)),
            _ => None,
        })
        .collect();
    // Whether each IN-subquery pin is itself bounded; the other pin sources need no evaluation.
    let pin_subqueries: Vec<Vec<Option<Evaluation>>> = query
        .items
        .iter()
        .map(|item| {
            item.pins
                .iter()
                .map(|pin| match &pin.source {
                    SqlPinSource::Query(inner) | SqlPinSource::ReadQuery(inner) => {
                        Some(evaluate(inner, catalog, precise_columns))
                    }
                    _ => None,
                })
                .collect()
        })
        .collect();
    let pin_arrays = super::arrays::pin_sources(query, catalog);
    let mut bounded: Vec<bool> = query
        .items
        .iter()
        .zip(&nested)
        .map(|(item, nested)| match &item.kind {
            // A table is bounded only by its pins, known to the catalog or not: an unknown one is
            // never reported, but it cannot size the relations pinned to it.
            SqlBoundItemKind::Table(_) => false,
            // A LATERAL source that reads earlier items is sized per row of them, so it bounds
            // nothing itself; the relations inside it are still judged below.
            SqlBoundItemKind::Query(_) => {
                !item.lateral
                    && !reads_outer(&item.lateral_reads, catalog)
                    && !qualified::reads_outer(&item.lateral_qualified_reads, catalog)
                    && nested.as_ref().is_some_and(|inner| inner.bounded)
            }
            SqlBoundItemKind::Other => true,
            // Nothing proves what it returns, so it sizes nothing pinned to it.
            SqlBoundItemKind::Opaque => false,
        })
        .collect();
    let keys: Vec<_> = query
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            keys::prepare(item, &pin_subqueries[index], &pin_arrays[index], catalog)
        })
        .collect();
    propagation::bound(&keys, &mut bounded);
    let offenders = reads::collect(
        query,
        &nested,
        &pin_subqueries,
        &bounded,
        catalog,
        precise_columns,
    );
    Evaluation {
        bounded: query.capped || bounded.iter().all(|state| *state),
        items: bounded,
        offenders,
    }
}

/// Columns every table has without the catalog listing them.
const SYSTEM_COLUMNS: &[&str] = &["ctid", "tableoid", "xmin", "xmax", "cmin", "cmax"];

/// Whether a subquery reads the query around it: a bare column that none of the base tables it
/// could belong to has, so PostgreSQL resolves it outward. A table the catalog does not
/// describe may have it, and is taken to.
fn reads_outer(reads: &[SqlBareRead], catalog: &SchemaCatalog) -> bool {
    reads.iter().any(|read| {
        !SYSTEM_COLUMNS.contains(&read.column.as_str())
            && read.tables.iter().all(|table| {
                catalog.relation(table).is_some_and(|table| {
                    table
                        .columns
                        .iter()
                        .all(|column| column.name != read.column)
                })
            })
    })
}
