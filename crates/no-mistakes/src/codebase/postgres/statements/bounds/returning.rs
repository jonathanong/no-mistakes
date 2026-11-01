//! Project a data-modifying CTE's RETURNING cardinality back to its target.
use crate::codebase::postgres::idents::{ident_key, unwrap_expr};
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundQuery, SqlPinSource};
use crate::codebase::postgres::SqlBoundItemKind;
use sqlparser::ast::{Expr, SelectItem};

pub(super) fn project(mut query: SqlBoundQuery) -> Option<SqlBoundQuery> {
    if query.items.is_empty() {
        return None;
    }
    let siblings = query.items.split_off(1);
    let mut sized = vec![false; siblings.len() + 1];
    for (offset, sibling) in siblings.iter().enumerate() {
        sized[offset + 1] = row_bounded(sibling);
    }
    let mut keep_item = vec![false; sized.len()];
    let mut keep_pin = Vec::with_capacity(query.items[0].pins.len());
    for pin in &query.items[0].pins {
        keep_pin.push(retain(&pin.source, &sized, &siblings, &mut keep_item));
    }
    let mut remap = vec![0; sized.len()];
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

/// Capped, caller-sized, or a target carried only by those inputs.
fn row_bounded(item: &SqlBoundItem) -> bool {
    match &item.kind {
        SqlBoundItemKind::Other => true,
        SqlBoundItemKind::Query(query) => query_bounded(query),
        _ => false,
    }
}

fn query_bounded(query: &SqlBoundQuery) -> bool {
    query.capped
        || (!query.items.is_empty() && query.items.iter().all(|item| item_sized(query, item)))
}

fn item_sized(query: &SqlBoundQuery, item: &SqlBoundItem) -> bool {
    match &item.kind {
        SqlBoundItemKind::Other => true,
        SqlBoundItemKind::Query(inner) => query_bounded(inner),
        SqlBoundItemKind::Table(_) => table_carried(query, item),
        _ => false,
    }
}

/// A value pin is not enough: `status = $1` does not size the projection.
fn table_carried(query: &SqlBoundQuery, item: &SqlBoundItem) -> bool {
    let mut carried = false;
    for pin in &item.pins {
        let Some(indexes) = sibling_indexes(&pin.source) else {
            continue;
        };
        if indexes
            .iter()
            .all(|index| query.items.get(*index).is_some_and(row_bounded))
        {
            carried = true;
        } else {
            return false;
        }
    }
    carried
}

fn retain(
    source: &SqlPinSource,
    sized: &[bool],
    siblings: &[SqlBoundItem],
    keep_item: &mut [bool],
) -> bool {
    let Some(indexes) = sibling_indexes(source) else {
        return matches!(source, SqlPinSource::Value | SqlPinSource::Query(_));
    };
    let sized_driver = indexes
        .iter()
        .all(|index| sized.get(*index).copied().unwrap_or(false));
    // Keep an opaque recordset so a later PK update names that input, not the target.
    let opaque_driver = indexes.iter().all(|index| {
        index
            .checked_sub(1)
            .and_then(|offset| siblings.get(offset))
            .is_some_and(opaque_driver)
    });
    if sized_driver || opaque_driver {
        for index in indexes {
            keep_item[index] = true;
        }
        return true;
    }
    false
}

fn opaque_driver(item: &SqlBoundItem) -> bool {
    match &item.kind {
        SqlBoundItemKind::Opaque => true,
        SqlBoundItemKind::Query(query) => query.has_opaque_row_inputs(),
        _ => false,
    }
}

pub(super) fn name_returning(query: &mut SqlBoundQuery, returning: &[SelectItem]) {
    if query.outputs.len() != returning.len() {
        return;
    }
    for (output, item) in query.outputs.iter_mut().zip(returning) {
        if output.name.is_some() {
            continue;
        }
        if let Some(name) = returning_name(item) {
            output.name = Some(name);
        }
    }
}

pub(super) fn returning_name(item: &SelectItem) -> Option<String> {
    match item {
        SelectItem::ExprWithAlias { alias, .. } => Some(ident_key(alias)),
        SelectItem::UnnamedExpr(expr) => match unwrap_expr(expr) {
            Expr::Identifier(ident) => Some(ident_key(ident)),
            Expr::CompoundIdentifier(parts) => parts.last().map(ident_key),
            _ => None,
        },
        _ => None,
    }
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
