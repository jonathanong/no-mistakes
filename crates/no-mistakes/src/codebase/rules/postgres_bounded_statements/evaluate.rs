//! Decide, against a schema catalog, whether a statement's rows are bounded.
use crate::codebase::postgres::statements::{
    SqlBoundFact, SqlBoundItem, SqlBoundItemKind, SqlBoundQuery, SqlPinSource,
};
use crate::codebase::postgres::{RelationKind, SchemaCatalog};

/// A catalog relation that a statement can read or change in unbounded numbers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Offender {
    /// The catalog's name for the table.
    pub(super) table: String,
    pub(super) line: usize,
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
    let evaluation = evaluate(&fact.query, catalog);
    let Some(target) = fact.target else {
        return evaluation.offenders;
    };
    // An UPDATE or DELETE changes each target row at most once, so only the target must be bounded.
    if fact.query.capped || evaluation.items[target] {
        return Vec::new();
    }
    let item = &fact.query.items[target];
    match &item.kind {
        SqlBoundItemKind::Table(name) => table_offender(name, item.line, catalog)
            .into_iter()
            .collect(),
        _ => Vec::new(),
    }
}

fn evaluate(query: &SqlBoundQuery, catalog: &SchemaCatalog) -> Evaluation {
    let nested: Vec<Option<Evaluation>> = query
        .items
        .iter()
        .map(|item| match &item.kind {
            SqlBoundItemKind::Query(inner) => Some(evaluate(inner, catalog)),
            _ => None,
        })
        .collect();
    // Whether each IN-subquery pin is itself bounded; the other pin sources need no evaluation.
    let pin_subqueries: Vec<Vec<bool>> = query
        .items
        .iter()
        .map(|item| {
            item.pins
                .iter()
                .map(|pin| match &pin.source {
                    SqlPinSource::Query(inner) => evaluate(inner, catalog).bounded,
                    _ => true,
                })
                .collect()
        })
        .collect();
    let mut bounded: Vec<bool> = query
        .items
        .iter()
        .zip(&nested)
        .map(|(item, nested)| match &item.kind {
            // A table is bounded only by its pins, known to the catalog or not: an unknown one is
            // never reported, but it cannot size the relations pinned to it.
            SqlBoundItemKind::Table(_) => false,
            SqlBoundItemKind::Query(_) => nested.as_ref().is_some_and(|inner| inner.bounded),
            SqlBoundItemKind::Other => true,
            // Nothing proves what it returns, so it sizes nothing pinned to it.
            SqlBoundItemKind::Opaque => false,
        })
        .collect();
    // Bounded items bound the relations pinned to them: iterate to the least fixed point.
    loop {
        let mut changed = false;
        for (index, item) in query.items.iter().enumerate() {
            if !bounded[index] && keyed(item, &pin_subqueries[index], &bounded, catalog) {
                bounded[index] = true;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let mut offenders = Vec::new();
    if !query.capped {
        for (index, item) in query.items.iter().enumerate() {
            if bounded[index] {
                continue;
            }
            if let Some(inner) = &nested[index] {
                offenders.extend(inner.offenders.iter().cloned());
            } else if let SqlBoundItemKind::Table(name) = &item.kind {
                offenders.extend(table_offender(name, item.line, catalog));
            }
        }
    }
    // The same relation can be reached by several arms; keep the first report of each.
    let mut seen = std::collections::HashSet::new();
    offenders.retain(|offender| seen.insert((offender.table.clone(), offender.line)));
    Evaluation {
        bounded: query.capped || bounded.iter().all(|state| *state),
        items: bounded,
        offenders,
    }
}

fn table_offender(name: &str, line: usize, catalog: &SchemaCatalog) -> Option<Offender> {
    Some(Offender {
        table: catalog.relation(name)?.name.clone(),
        line,
    })
}

/// Whether the pins cover every column of a unique key (or the row identifier) with values
/// the statement or its caller sizes.
fn keyed(
    item: &SqlBoundItem,
    subqueries: &[bool],
    bounded: &[bool],
    catalog: &SchemaCatalog,
) -> bool {
    let SqlBoundItemKind::Table(name) = &item.kind else {
        return false;
    };
    let usable = |column: &str| {
        item.pins.iter().enumerate().any(|(index, pin)| {
            pin.column == column
                // `IS NOT DISTINCT FROM $1` also matches NULL, which a unique key may repeat.
                && (!pin.null_safe || catalog.column_is_not_null(name, column))
                && match &pin.source {
                    SqlPinSource::Value => true,
                    SqlPinSource::Items(items) => items.iter().all(|other| bounded[*other]),
                    SqlPinSource::Query(_) => subqueries[index],
                }
        })
    };
    let mut keys = catalog.unique_keys(name);
    // Every row has a `ctid`, so `ctid IN (SELECT ctid … LIMIT n)` bounds a statement. It is
    // only unique within one physical table: the leaves of a partitioned table repeat values.
    let partitioned = catalog
        .relation(name)
        .is_some_and(|table| table.relation_kind == RelationKind::PartitionedTable);
    if !partitioned {
        keys.push(vec!["ctid".to_string()]);
    }
    keys.iter()
        .any(|key| key.iter().all(|column| usable(column)))
}
