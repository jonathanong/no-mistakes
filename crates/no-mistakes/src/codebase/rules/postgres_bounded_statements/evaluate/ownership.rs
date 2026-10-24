//! Resolve bare pin candidates only when the complete catalog identifies one visible owner.
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundItemKind};
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
        let SqlBoundItemKind::Table(name) = &item.kind else {
            return BTreeMap::new();
        };
        // A positional alias changes visible names; an opaque or absent competitor may own any
        // bare column. Both prevent a sound ownership proof.
        if !item.column_aliases.is_empty() {
            return BTreeMap::new();
        }
        let Some(table) = catalog.relation(name) else {
            return BTreeMap::new();
        };
        for column in &table.columns {
            if !candidates.contains(&column.name) {
                continue;
            }
            if owners.insert(column.name.clone(), index).is_some() {
                ambiguous.insert(column.name.clone());
            }
        }
    }
    owners.retain(|column, _| !ambiguous.contains(column));
    owners
}
