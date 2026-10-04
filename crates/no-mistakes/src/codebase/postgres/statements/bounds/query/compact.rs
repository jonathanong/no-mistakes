use super::super::items;
use crate::codebase::postgres::statements::{
    SqlBoundInputMode, SqlBoundItem, SqlBoundItemKind, SqlBoundQuery, SqlPinSource,
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
    fn tables(
        query: &SqlBoundQuery,
        mut mandatory: bool,
        mut blocking_only: bool,
        found: &mut BTreeMap<String, (SqlBoundItem, bool)>,
    ) {
        if query.input_mode == SqlBoundInputMode::Skipped {
            return;
        }
        if query.capped {
            mandatory = false;
            blocking_only = true;
        } else if query.input_mode == SqlBoundInputMode::Blocking {
            mandatory = true;
            blocking_only = false;
        }
        for item in &query.items {
            // Discard pin eligibility separately from reads the pin expression executes.
            for pin in &item.pins {
                if let SqlPinSource::Query(inner) | SqlPinSource::ReadQuery(inner) = &pin.source {
                    tables(inner, mandatory, blocking_only, found);
                }
            }
            match &item.kind {
                SqlBoundItemKind::Table(name) if !blocking_only || mandatory => {
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
                    if let Some((previous, blocking)) = found.get_mut(name) {
                        previous.pins.retain(|pin| pins.contains(pin));
                        *blocking |= mandatory;
                    } else {
                        let mut table = SqlBoundItem::new(
                            SqlBoundItemKind::Table(name.clone()),
                            None,
                            (item.line, item.column),
                        );
                        table.pins = pins;
                        found.insert(name.clone(), (table, mandatory));
                    }
                }
                SqlBoundItemKind::Query(inner) => tables(inner, mandatory, blocking_only, found),
                _ => {}
            }
        }
    }
    let mut found = BTreeMap::new();
    tables(bound, false, false, &mut found);
    let mut items: Vec<_> = found
        .into_values()
        .map(|(item, mandatory)| {
            if !mandatory {
                return item;
            }
            let at = (item.line, item.column);
            SqlBoundItem::new(
                SqlBoundItemKind::Query(SqlBoundQuery {
                    input_mode: SqlBoundInputMode::Blocking,
                    capped: false,
                    items: vec![item],
                    outputs: Vec::new(),
                }),
                None,
                at,
            )
        })
        .collect();
    items.push(items::opaque(at));
    SqlBoundQuery {
        input_mode: bound.input_mode,
        outputs: bound.outputs.clone(),
        capped: bound.capped,
        items,
    }
}
