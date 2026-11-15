//! Decide, against a schema catalog, whether a statement's rows are bounded.
use super::view_bounds::{ViewBoundMode, ViewBounds};
use crate::codebase::postgres::statements::{
    SqlBoundFact, SqlBoundInputMode, SqlBoundItemKind, SqlBoundQuery,
};
use crate::codebase::postgres::{CatalogObjectRef, SchemaCatalog};
mod accepts;
pub(in crate::codebase::rules::postgres_bounded_statements) use accepts::DomainAccept;
mod body_reads;
mod correlated;
mod driven;
mod keys;
mod lateral;
pub(super) mod lookup;
mod outer_reads;
mod ownership;
mod possible_temporary;
mod propagation;
mod qualified;
mod reads;
mod scalar_output;
mod scalar_reads;
mod settle;
mod singleton;
mod statement;
mod target;
mod view_key;
use outer_reads::reads_outer;
pub(super) use view_key::visible_key;

/// A catalog relation that a statement can read or change in unbounded numbers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Offender {
    /// The catalog's name for the table.
    pub(super) table: String,
    pub(super) object: CatalogObjectRef,
    pub(super) line: usize,
    pub(super) column: usize,
    blocking_input: bool,
    /// Unique-key domain product that exceeded `maxDomainRows`.
    pub(super) domain_over: Option<u64>,
}

struct Evaluation {
    /// A `LIMIT`, a pure aggregate, or every FROM item bounded.
    bounded: bool,
    /// Per FROM item: whether that item is bounded.
    items: Vec<bool>,
    finite_outputs: Option<Vec<bool>>,
    offenders: Vec<Offender>,
    credits: Vec<Option<keys::Credit>>,
    accepts: Vec<accepts::DomainAccept>,
}

/// The relations that make `fact` unbounded. A relation the catalog does not know is not
/// judged, and it bounds nothing either: it can supply every value of a column pinned to it.
#[cfg(any(test, feature = "test-instrumentation"))]
pub(super) fn offenders(fact: &SqlBoundFact, catalog: &SchemaCatalog) -> Vec<Offender> {
    statement::judgment(fact, catalog, &ViewBounds::default(), false).offenders
}

pub(super) fn offenders_with_views(
    fact: &SqlBoundFact,
    catalog: &SchemaCatalog,
    views: &ViewBounds,
) -> Vec<Offender> {
    statement::judgment(fact, catalog, views, false).offenders
}

pub(super) fn variant_offenders_with_views(
    fact: &SqlBoundFact,
    catalog: &SchemaCatalog,
    views: &ViewBounds,
) -> Vec<Offender> {
    statement::judgment(fact, catalog, views, true).offenders
}

pub(in crate::codebase::rules::postgres_bounded_statements) use statement::{
    judgment_with_views, variant_judgment_with_views, Judgment,
};

fn evaluate(
    query: &SqlBoundQuery,
    catalog: &SchemaCatalog,
    views: &ViewBounds,
    precise_columns: bool,
    write_target: Option<usize>,
) -> Evaluation {
    if query.input_mode == SqlBoundInputMode::Skipped {
        return Evaluation {
            bounded: true,
            items: vec![true; query.items.len()],
            finite_outputs: None,
            offenders: Vec::new(),
            credits: vec![None; query.items.len()],
            accepts: Vec::new(),
        };
    }
    let nested: Vec<Option<Evaluation>> = query
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            correlated::nested(item, &query.items[..index], catalog, views, precise_columns)
        })
        .collect();
    let pin_subqueries = correlated::pin_queries(query, catalog, views, precise_columns);
    let pin_arrays = super::arrays::pin_sources(query, catalog);
    let owners = ownership::owners(&query.items, catalog);
    let mut bounded: Vec<bool> = query
        .items
        .iter()
        .zip(&nested)
        .map(|(item, nested)| match &item.kind {
            // A table is bounded only by its pins, known to the catalog or not: an unknown one is
            // never reported, but it cannot size the relations pinned to it.
            SqlBoundItemKind::Table(name) => views
                .get(name, catalog)
                .is_some_and(|(_, mode)| mode == ViewBoundMode::Bounded),
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
    let write_target =
        write_target.or((query.input_mode == SqlBoundInputMode::Returning).then_some(0));
    let domains = if write_target.is_some() {
        keys::finite_domains(query, &nested)
    } else {
        Default::default()
    };
    let mut credits = Vec::new();
    let keys: Vec<_> = query
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            if nested[index].as_ref().is_some_and(|inner| inner.bounded) {
                if let Some(drivers) = lateral::dependencies(item, &query.items[..index], catalog) {
                    credits.push(None);
                    return keys::dependent_on(drivers);
                }
            }
            let (prepared, credit) = keys::prepare(
                item,
                index,
                (&owners, (write_target == Some(index)).then_some(&domains)),
                &pin_subqueries[index],
                &pin_arrays[index],
                catalog,
                views,
            );
            credits.push(credit);
            prepared
        })
        .collect();
    let (result_bounded, offenders) = settle::finish(
        &settle::Inputs {
            query,
            keys: &keys,
            nested: &nested,
            pin_subqueries: &pin_subqueries,
            catalog,
            views,
            precise_columns,
            credits: &credits,
        },
        &mut bounded,
    );
    let accepts = accepts::collect(
        query,
        &bounded,
        &credits,
        &nested,
        &pin_subqueries,
        catalog,
        views.domain_rows,
    );
    target::project(
        query,
        Evaluation {
            bounded: result_bounded,
            finite_outputs: keys::output_domains(query, &nested),
            items: bounded,
            offenders,
            credits,
            accepts,
        },
        catalog,
        views,
    )
}
