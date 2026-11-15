//! A validated all-NOT-NULL foreign key copies the remaining columns from one referenced row.
use super::peer;
use crate::codebase::postgres::statements::{SqlBoundItemKind, SqlBoundPin, SqlBoundQuery};
use crate::codebase::postgres::SchemaCatalog;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn imply(
    query: &SqlBoundQuery,
    catalog: &SchemaCatalog,
    columns: &mut [BTreeSet<String>],
) {
    for (index, item) in query.items.iter().enumerate() {
        let SqlBoundItemKind::Table(name) = &item.kind else {
            continue;
        };
        let Some(table) = catalog.bounded_relation(name) else {
            continue;
        };
        for key in &table.foreign_keys {
            if !key.validated
                || key.columns.is_empty()
                || key.columns.len() != key.referenced_columns.len()
                || key
                    .columns
                    .iter()
                    .any(|column| !catalog.column_is_not_null(name, column))
            {
                continue;
            }
            let mut equated: BTreeMap<usize, BTreeSet<String>> = BTreeMap::new();
            for (local, remote) in key.columns.iter().zip(&key.referenced_columns) {
                for pin in &item.pins {
                    let Some(source) = equated_source(pin, local, remote) else {
                        continue;
                    };
                    if source == index || !same_table(query, catalog, source, &key.referenced_table)
                    {
                        continue;
                    }
                    equated.entry(source).or_default().insert(remote.clone());
                }
            }
            for (source, remote_columns) in equated {
                let Some(known) = columns.get(source) else {
                    continue;
                };
                let Some(source_item) = query.items.get(source) else {
                    continue;
                };
                let SqlBoundItemKind::Table(source_name) = &source_item.kind else {
                    continue;
                };
                let Some(source_table) = catalog.bounded_relation(source_name) else {
                    continue;
                };
                let equated: BTreeSet<_> = remote_columns
                    .iter()
                    .filter(|column| known.contains(column.as_str()))
                    .cloned()
                    .collect();
                let pinned =
                    super::physical_pinned(source_table, &source_item.column_aliases, &equated);
                if !catalog.columns_pin_one_row(source_name, &pinned) {
                    continue;
                }
                for column in &key.columns {
                    columns[index].insert(column.clone());
                }
            }
        }
    }
}

fn equated_source(pin: &SqlBoundPin, local: &str, remote: &str) -> Option<usize> {
    if pin.column != local || pin.peer_column.as_deref() != Some(remote) {
        return None;
    }
    peer(pin)
}

fn same_table(
    query: &SqlBoundQuery,
    catalog: &SchemaCatalog,
    source: usize,
    referenced: &str,
) -> bool {
    let Some(SqlBoundItemKind::Table(name)) = query.items.get(source).map(|item| &item.kind) else {
        return false;
    };
    let Some(local) = catalog.bounded_relation(name) else {
        return false;
    };
    catalog
        .bounded_relation(referenced)
        .is_some_and(|remote| remote.name == local.name)
}
