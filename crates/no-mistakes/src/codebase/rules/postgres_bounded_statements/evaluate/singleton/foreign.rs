//! A validated all-NOT-NULL foreign key copies the remaining columns from one referenced row.
use super::peer;
use crate::codebase::postgres::statements::{SqlBoundItemKind, SqlBoundQuery};
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
                let Some(local_visible) = visible(table, &item.column_aliases, local) else {
                    continue;
                };
                for pin in &item.pins {
                    let Some((source, peer_column)) = peer(pin) else {
                        continue;
                    };
                    if pin.column != local_visible {
                        continue;
                    }
                    let Some(source_table) =
                        same_table(query, catalog, source, &key.referenced_table)
                    else {
                        continue;
                    };
                    let Some(remote_visible) =
                        visible(source_table, &query.items[source].column_aliases, remote)
                    else {
                        continue;
                    };
                    if peer_column == remote_visible {
                        equated
                            .entry(source)
                            .or_default()
                            .insert(remote_visible.to_owned());
                    }
                }
            }
            for (source, remote_columns) in equated {
                if let Some(source_table) =
                    same_table(query, catalog, source, &key.referenced_table)
                {
                    let known = &columns[source];
                    let equated_visible: BTreeSet<_> = remote_columns
                        .iter()
                        .filter(|column| known.contains(column.as_str()))
                        .cloned()
                        .collect();
                    let pinned = super::physical_pinned(
                        source_table,
                        &query.items[source].column_aliases,
                        &equated_visible,
                    );
                    if catalog.columns_pin_one_row(&source_table.name, &pinned) {
                        for column in &key.columns {
                            if let Some(name) = visible(table, &item.column_aliases, column) {
                                columns[index].insert(name.to_owned());
                            }
                        }
                    }
                }
            }
        }
    }
}

fn visible<'a>(
    table: &'a crate::codebase::postgres::CatalogTable,
    aliases: &'a [String],
    column: &'a str,
) -> Option<&'a str> {
    super::super::super::arrays::key_visible(table, aliases, column)
}

fn same_table<'a>(
    query: &'a SqlBoundQuery,
    catalog: &'a SchemaCatalog,
    source: usize,
    referenced: &str,
) -> Option<&'a crate::codebase::postgres::CatalogTable> {
    let Some(SqlBoundItemKind::Table(name)) = query.items.get(source).map(|item| &item.kind) else {
        return None;
    };
    let local = catalog.bounded_relation(name)?;
    catalog
        .bounded_relation(referenced)
        .is_some_and(|remote| remote.name == local.name)
        .then_some(local)
}
