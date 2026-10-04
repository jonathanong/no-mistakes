use super::super::items;
use crate::codebase::postgres::statements::{
    SqlBoundItem, SqlBoundItemKind, SqlBoundQuery, SqlPinSource,
};
use std::collections::BTreeMap;

pub(super) const MAX_BOUND_ITEMS: usize = 2048;

/// The number of items in a bound, nested queries and subquery pins included.
pub(super) fn size(query: &SqlBoundQuery) -> usize {
    query
        .items
        .iter()
        .map(|item| {
            let inner = match &item.kind {
                SqlBoundItemKind::Query(inner) => size(inner),
                _ => 0,
            };
            let pins: usize = item
                .pins
                .iter()
                .map(|pin| match &pin.source {
                    SqlPinSource::Query(inner) | SqlPinSource::ReadQuery(inner) => size(inner),
                    _ => 0,
                })
                .sum();
            1 + inner + pins
        })
        .sum()
}

/// Keep a fail-closed summary of oversized CTEs. Capped subqueries contribute no
/// unbounded reads; an opaque item prevents this summary from sizing a later join.
pub(super) fn compact(bound: &SqlBoundQuery, at: (usize, usize)) -> SqlBoundQuery {
    fn tables(query: &SqlBoundQuery, found: &mut BTreeMap<String, SqlBoundItem>) {
        if query.capped {
            return;
        }
        for item in &query.items {
            // Discard pin eligibility separately from reads the pin expression executes.
            for pin in &item.pins {
                if let SqlPinSource::Query(inner) | SqlPinSource::ReadQuery(inner) = &pin.source {
                    tables(inner, found);
                }
            }
            match &item.kind {
                SqlBoundItemKind::Table(name) => {
                    // Only caller-sized values remain valid after the surrounding items are
                    // removed. Positional aliases cannot keep catalog-key credit once their
                    // identity is discarded. Repeated reads retain only their common pins.
                    let pins: Vec<_> = item
                        .pins
                        .iter()
                        .filter(|pin| {
                            item.column_aliases.is_empty()
                                && super::super::compact_pins::independent(&pin.source)
                        })
                        .cloned()
                        .collect();
                    if let Some(previous) = found.get_mut(name) {
                        previous.pins.retain(|pin| pins.contains(pin));
                    } else {
                        let mut table = SqlBoundItem::new(
                            SqlBoundItemKind::Table(name.clone()),
                            None,
                            (item.line, item.column),
                        );
                        table.pins = pins;
                        found.insert(name.clone(), table);
                    }
                }
                SqlBoundItemKind::Query(inner) => tables(inner, found),
                _ => {}
            }
        }
    }
    let mut found = BTreeMap::new();
    tables(bound, &mut found);
    let mut items: Vec<_> = found.into_values().collect();
    items.push(items::opaque(at));
    SqlBoundQuery {
        outputs: bound.outputs.clone(),
        capped: bound.capped,
        items,
    }
}
