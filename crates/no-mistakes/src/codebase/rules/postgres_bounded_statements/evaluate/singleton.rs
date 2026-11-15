//! One-row columns: unique-key rows, direct column equalities, and validated foreign keys.
mod foreign;

use super::keys::Keys;
use super::propagation;
use crate::codebase::postgres::statements::{
    SqlBoundItemKind, SqlBoundPin, SqlBoundQuery, SqlPinSource,
};
use crate::codebase::postgres::SchemaCatalog;
use std::collections::BTreeSet;

const STEPS: usize = 32;

pub(super) fn finish(
    query: &SqlBoundQuery,
    catalog: &SchemaCatalog,
    keys: &[Keys],
    bounded: &mut [bool],
) {
    let mut columns = single_columns(query);
    for _ in 0..STEPS {
        let before_bounded = bounded.to_vec();
        let before_columns = columns.clone();
        transfer(query, &mut columns);
        rows(query, catalog, &mut columns, bounded);
        foreign::imply(query, catalog, &mut columns);
        rows(query, catalog, &mut columns, bounded);
        propagation::bound(keys, bounded);
        if bounded == before_bounded.as_slice() && columns == before_columns {
            return;
        }
    }
}

fn single_columns(query: &SqlBoundQuery) -> Vec<BTreeSet<String>> {
    query
        .items
        .iter()
        .map(|item| {
            item.pins
                .iter()
                .filter(|pin| pin.single_value && unconditional(pin))
                .map(|pin| pin.column.clone())
                .collect()
        })
        .collect()
}

fn transfer(query: &SqlBoundQuery, columns: &mut [BTreeSet<String>]) {
    for (index, item) in query.items.iter().enumerate() {
        for pin in &item.pins {
            let Some((source, peer_column)) = peer(pin) else {
                continue;
            };
            if source != index
                && columns
                    .get(source)
                    .is_some_and(|set| set.contains(peer_column))
            {
                columns[index].insert(pin.column.clone());
            }
        }
    }
}

fn rows(
    query: &SqlBoundQuery,
    catalog: &SchemaCatalog,
    columns: &mut [BTreeSet<String>],
    bounded: &mut [bool],
) {
    for (index, item) in query.items.iter().enumerate() {
        let SqlBoundItemKind::Table(name) = &item.kind else {
            continue;
        };
        let Some(table) = catalog.bounded_relation(name) else {
            continue;
        };
        let pinned = physical_pinned(table, &item.column_aliases, &columns[index]);
        if !catalog.columns_pin_one_row(name, &pinned) {
            continue;
        }
        bounded[index] = true;
        for column in &table.columns {
            if let Some(visible) =
                super::super::arrays::key_visible(table, &item.column_aliases, &column.name)
            {
                columns[index].insert(visible.to_owned());
            }
        }
    }
}

/// Visible names that still denote these physical columns. A positional alias that
/// renames another column to a key's spelling does not satisfy that key.
pub(super) fn physical_pinned(
    table: &crate::codebase::postgres::CatalogTable,
    aliases: &[String],
    visible: &BTreeSet<String>,
) -> Vec<String> {
    table
        .columns
        .iter()
        .filter_map(|column| {
            let name = super::super::arrays::key_visible(table, aliases, &column.name)?;
            visible.contains(name).then(|| column.name.clone())
        })
        .collect()
}

pub(super) fn unconditional(pin: &SqlBoundPin) -> bool {
    pin.alternative.is_none()
        && pin.expression.is_none()
        && !pin.null_safe
        && !pin.unqualified_candidate
        && pin.reads.is_empty()
        && pin.qualified_reads.is_empty()
}

fn peer(pin: &SqlBoundPin) -> Option<(usize, &str)> {
    if !unconditional(pin) {
        return None;
    }
    let SqlPinSource::Items(sources) = &pin.source else {
        return None;
    };
    let [source] = sources.as_slice() else {
        return None;
    };
    pin.peer_column.as_deref().map(|column| (*source, column))
}
