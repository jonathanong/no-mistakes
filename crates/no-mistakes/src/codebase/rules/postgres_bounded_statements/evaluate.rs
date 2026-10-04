//! Decide, against a schema catalog, whether a statement's rows are bounded.
use crate::codebase::postgres::statements::{
    SqlBareRead, SqlBoundFact, SqlBoundItem, SqlBoundItemKind, SqlBoundQuery, SqlPinSource,
};
use crate::codebase::postgres::{RelationKind, SchemaCatalog};

mod possible_temporary;

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
    let query = possible_temporary::project(&fact.query, catalog);
    let evaluation = evaluate(&query, catalog);
    let Some(target) = fact.target else {
        return evaluation.offenders;
    };
    // An UPDATE or DELETE changes each target row at most once, so only the target must be bounded.
    if query.capped || evaluation.items[target] {
        return Vec::new();
    }
    let item = &query.items[target];
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
    let pin_subqueries: Vec<Vec<Option<Evaluation>>> = query
        .items
        .iter()
        .map(|item| {
            item.pins
                .iter()
                .map(|pin| match &pin.source {
                    SqlPinSource::Query(inner) => Some(evaluate(inner, catalog)),
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
                    && nested.as_ref().is_some_and(|inner| inner.bounded)
            }
            SqlBoundItemKind::Other => true,
            // Nothing proves what it returns, so it sizes nothing pinned to it.
            SqlBoundItemKind::Opaque => false,
        })
        .collect();
    // Bounded items bound the relations pinned to them: iterate to the least fixed point.
    loop {
        let mut changed = false;
        for (index, item) in query.items.iter().enumerate() {
            if !bounded[index]
                && keyed(
                    item,
                    &pin_subqueries[index],
                    &pin_arrays[index],
                    &bounded,
                    catalog,
                )
            {
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
            for pin in pin_subqueries[index].iter().flatten() {
                offenders.extend(pin.offenders.iter().cloned());
            }
            if let Some(inner) = &nested[index] {
                offenders.extend(inner.offenders.iter().cloned());
            }
            if bounded[index] {
                continue;
            }
            if nested[index].is_none() {
                if let SqlBoundItemKind::Table(name) = &item.kind {
                    offenders.extend(table_offender(name, item.line, catalog));
                }
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
    subqueries: &[Option<Evaluation>],
    arrays: &[bool],
    bounded: &[bool],
    catalog: &SchemaCatalog,
) -> bool {
    let SqlBoundItemKind::Table(name) = &item.kind else {
        return false;
    };
    let usable = |column: &str, physical: &str| {
        item.pins.iter().enumerate().any(|(index, pin)| {
            pin.column == column
                && arrays[index]
                // A subquery in the value that reads the row checked sizes nothing.
                && !reads_outer(&pin.reads, catalog)
                // `IS NOT DISTINCT FROM $1` also matches NULL, which a unique key may repeat.
                && (!pin.null_safe || catalog.column_is_not_null(name, physical))
                && match &pin.source {
                    SqlPinSource::Value => true,
                    SqlPinSource::StoredArray(_) => false,
                    SqlPinSource::Items(items) | SqlPinSource::Array { items, .. } => items.iter().all(|other| bounded[*other]),
                    SqlPinSource::Query(_) => subqueries[index]
                        .as_ref()
                        .is_some_and(|query| query.bounded),
                }
        })
    };
    // Preserve physical nullability while matching pins against visible aliases.
    let mut keys: Vec<_> = catalog
        .unique_keys(name)
        .into_iter()
        .filter_map(|key| {
            key.into_iter()
                .map(|column| {
                    let table = catalog.relation(name)?;
                    let visible = super::arrays::key_visible(table, &item.column_aliases, &column)?;
                    Some((visible.to_owned(), column))
                })
                .collect::<Option<Vec<_>>>()
        })
        .collect();
    // Every row has a `ctid`, so `ctid IN (SELECT ctid … LIMIT n)` bounds a statement. It is
    // only unique within one physical table: the leaves of a partitioned table repeat values,
    // and a relation the catalog does not describe may be one.
    let plain = catalog
        .relation(name)
        .is_some_and(|table| table.relation_kind != RelationKind::PartitionedTable);
    if plain && !item.column_aliases.iter().any(|alias| alias == "ctid") {
        keys.push(vec![("ctid".to_string(), "ctid".to_string())]);
    }
    keys.iter().any(|key| {
        key.iter()
            .all(|(visible, physical)| usable(visible, physical))
    })
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
