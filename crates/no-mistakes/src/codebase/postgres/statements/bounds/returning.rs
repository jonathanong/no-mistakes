//! Project a data-modifying CTE's RETURNING cardinality back to its target.
use crate::codebase::postgres::statements::{SqlBoundQuery, SqlPinSource};
use crate::codebase::postgres::SqlBoundItemKind;

pub(super) fn project(mut query: SqlBoundQuery) -> Option<SqlBoundQuery> {
    if query.items.is_empty() {
        return None;
    }
    let siblings = query.items.split_off(1);
    let mut capped = vec![false; siblings.len() + 1];
    for (offset, sibling) in siblings.iter().enumerate() {
        capped[offset + 1] =
            matches!(&sibling.kind, SqlBoundItemKind::Query(bound) if bound.capped);
    }
    let mut keep_item = vec![false; capped.len()];
    let mut keep_pin = Vec::with_capacity(query.items[0].pins.len());
    for pin in &query.items[0].pins {
        keep_pin.push(retain(&pin.source, &capped, &mut keep_item));
    }
    let mut remap = vec![0; capped.len()];
    let mut kept = Vec::new();
    for (offset, sibling) in siblings.into_iter().enumerate() {
        let index = offset + 1;
        if keep_item[index] {
            remap[index] = 1 + kept.len();
            kept.push(sibling);
        }
    }
    let pins = std::mem::take(&mut query.items[0].pins);
    query.items[0].pins = pins
        .into_iter()
        .zip(keep_pin)
        .filter_map(|(mut pin, keep)| {
            if !keep {
                return None;
            }
            rewrite(&mut pin.source, &remap);
            Some(pin)
        })
        .collect();
    query.items.extend(kept);
    query.outputs.clear();
    Some(query)
}

fn retain(source: &SqlPinSource, capped: &[bool], keep_item: &mut [bool]) -> bool {
    let Some(indexes) = sibling_indexes(source) else {
        return matches!(source, SqlPinSource::Value | SqlPinSource::Query(_));
    };
    if indexes
        .iter()
        .all(|index| capped.get(*index).copied().unwrap_or(false))
    {
        for index in indexes {
            keep_item[index] = true;
        }
        return true;
    }
    false
}

fn sibling_indexes(source: &SqlPinSource) -> Option<Vec<usize>> {
    match source {
        SqlPinSource::Items(items) if !items.is_empty() => Some(items.clone()),
        SqlPinSource::Array {
            items,
            scalar_columns,
            indexed_columns,
            ..
        } if !items.is_empty() => {
            let mut indexes = items.clone();
            indexes.extend(scalar_columns.iter().map(|(index, _)| *index));
            indexes.extend(indexed_columns.iter().map(|(index, _)| *index));
            Some(indexes)
        }
        _ => None,
    }
}

fn rewrite(source: &mut SqlPinSource, remap: &[usize]) {
    match source {
        SqlPinSource::Items(items) => apply(items, remap),
        SqlPinSource::Array {
            items,
            scalar_columns,
            indexed_columns,
            ..
        } => {
            apply(items, remap);
            for (index, _) in scalar_columns.iter_mut().chain(indexed_columns.iter_mut()) {
                *index = remap[*index];
            }
        }
        _ => {}
    }
}

fn apply(items: &mut [usize], remap: &[usize]) {
    for index in items {
        *index = remap[*index];
    }
}
