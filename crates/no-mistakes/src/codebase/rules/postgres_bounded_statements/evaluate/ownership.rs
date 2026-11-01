//! Resolve bare pin candidates only when the complete catalog identifies one visible owner.
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundItemKind, SqlBoundQuery};
use crate::codebase::postgres::{CatalogCoverage, SchemaCatalog};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn owners(items: &[SqlBoundItem], catalog: &SchemaCatalog) -> BTreeMap<String, usize> {
    let candidates: BTreeSet<_> = items
        .iter()
        .flat_map(|item| item.pins.iter())
        .filter(|pin| pin.unqualified_candidate)
        .map(|pin| pin.column.clone())
        .collect();
    if candidates.is_empty() || catalog.coverage() != CatalogCoverage::Complete {
        return BTreeMap::new();
    }
    let mut owners = BTreeMap::new();
    let mut ambiguous = BTreeSet::new();
    for (index, item) in items.iter().enumerate() {
        // A positional alias on a base table renames every catalog column. An opaque item, or a
        // derived item whose outputs are not a complete name list, may own any bare column.
        let Some(names) = visible_columns(item, catalog) else {
            return BTreeMap::new();
        };
        for column in &names {
            if !candidates.contains(column) {
                continue;
            }
            if owners.insert(column.clone(), index).is_some() {
                ambiguous.insert(column.clone());
            }
        }
    }
    owners.retain(|column, _| !ambiguous.contains(column));
    owners
}

fn visible_columns(item: &SqlBoundItem, catalog: &SchemaCatalog) -> Option<BTreeSet<String>> {
    match &item.kind {
        SqlBoundItemKind::Table(name) => {
            if !item.column_aliases.is_empty() {
                return None;
            }
            let table = catalog.bounded_relation(name)?;
            Some(
                table
                    .columns
                    .iter()
                    .map(|column| column.name.clone())
                    .collect(),
            )
        }
        SqlBoundItemKind::Query(query) => query_columns(item, query),
        SqlBoundItemKind::Other | SqlBoundItemKind::Opaque => None,
    }
}

/// Column aliases are the complete visible list. Otherwise every output needs a name:
/// a wildcard or an unnamed expression can still hide the bare column.
fn query_columns(item: &SqlBoundItem, query: &SqlBoundQuery) -> Option<BTreeSet<String>> {
    if !item.column_aliases.is_empty() {
        return Some(item.column_aliases.iter().cloned().collect());
    }
    if query.outputs.is_empty() {
        return None;
    }
    query
        .outputs
        .iter()
        .map(|output| output.name.clone())
        .collect()
}
