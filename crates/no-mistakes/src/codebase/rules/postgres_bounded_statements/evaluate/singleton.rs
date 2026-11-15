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
            let Some(source) = peer(pin) else {
                continue;
            };
            if source == index {
                continue;
            }
            let Some(peer_column) = pin.peer_column.as_deref() else {
                continue;
            };
            if columns
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
        let pinned: Vec<_> = columns[index].iter().cloned().collect();
        if !catalog.columns_pin_one_row(name, &pinned) {
            continue;
        }
        bounded[index] = true;
        for column in &table.columns {
            columns[index].insert(column.name.clone());
        }
    }
}

pub(super) fn unconditional(pin: &SqlBoundPin) -> bool {
    pin.alternative.is_none()
        && pin.expression.is_none()
        && !pin.null_safe
        && !pin.unqualified_candidate
        && pin.reads.is_empty()
        && pin.qualified_reads.is_empty()
}

fn peer(pin: &SqlBoundPin) -> Option<usize> {
    if !unconditional(pin) {
        return None;
    }
    let SqlPinSource::Items(sources) = &pin.source else {
        return None;
    };
    let [source] = sources.as_slice() else {
        return None;
    };
    pin.peer_column.is_some().then_some(*source)
}
