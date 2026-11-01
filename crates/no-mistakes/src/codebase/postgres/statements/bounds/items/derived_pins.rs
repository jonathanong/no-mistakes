//! Follow a derived item's output pin into the base column that projects it.
use crate::codebase::postgres::statements::{
    SqlBoundItem, SqlBoundItemKind, SqlBoundPin, SqlBoundQuery, SqlPinSource,
};
use std::collections::BTreeMap;

pub(super) fn push(items: &mut [SqlBoundItem]) {
    for item in items {
        if !matches!(item.kind, SqlBoundItemKind::Query(_)) {
            continue;
        }
        let alternatives = item.pin_alternatives;
        let aliases = item.column_aliases.clone();
        let pins: Vec<SqlBoundPin> = item
            .pins
            .iter()
            .filter(|pin| portable(pin))
            .cloned()
            .collect();
        if !pins.is_empty() {
            if let SqlBoundItemKind::Query(query) = &mut item.kind {
                let mut planned: BTreeMap<Vec<usize>, Vec<(String, SqlBoundPin)>> = BTreeMap::new();
                for pin in &pins {
                    let Some(ordinal) = ordinal(query, &aliases, &pin.column) else {
                        continue;
                    };
                    let mut path = Vec::new();
                    plan(query, ordinal, pin, &mut path, &mut planned);
                }
                apply(query, alternatives, &planned);
            }
        }
        // Copied onto the base column. A pin left here is not a separate bound.
        item.pins.clear();
    }
}

fn portable(pin: &SqlBoundPin) -> bool {
    // Item indexes belong to the outer statement. Only self-contained sources can move inward.
    !pin.unqualified_candidate
        && matches!(
            pin.source,
            SqlPinSource::Value
                | SqlPinSource::Query(_)
                | SqlPinSource::ReadQuery(_)
                | SqlPinSource::ArrayQuery(_)
        )
}

fn ordinal(query: &SqlBoundQuery, aliases: &[String], column: &str) -> Option<usize> {
    let names: Vec<Option<&str>> = if !aliases.is_empty() {
        aliases.iter().map(|name| Some(name.as_str())).collect()
    } else if query.outputs.is_empty() {
        return None;
    } else {
        query
            .outputs
            .iter()
            .map(|output| output.name.as_deref())
            .collect()
    };
    let mut found = names
        .iter()
        .enumerate()
        .filter(|(_, name)| name.as_deref() == Some(column));
    let (index, _) = found.next()?;
    found.next().is_none().then_some(index)
}

fn plan(
    query: &SqlBoundQuery,
    output: usize,
    pin: &SqlBoundPin,
    path: &mut Vec<usize>,
    planned: &mut BTreeMap<Vec<usize>, Vec<(String, SqlBoundPin)>>,
) {
    if let Some((index, column)) = query
        .outputs
        .get(output)
        .and_then(|output| output.source_column.clone())
    {
        let table = matches!(
            query.items.get(index).map(|item| &item.kind),
            Some(SqlBoundItemKind::Table(_))
        );
        if table {
            path.push(index);
            planned
                .entry(path.clone())
                .or_default()
                .push((column, pin.clone()));
            path.pop();
            return;
        }
        let Some((inner, next)) = projected_query(query, index, &column) else {
            return;
        };
        path.push(index);
        plan(inner, next, pin, path, planned);
        path.pop();
        return;
    }
    if !set_operation(query) || query.outputs.len() <= output {
        return;
    }
    for index in 0..query.items.len() {
        if let SqlBoundItemKind::Query(inner) = &query.items[index].kind {
            path.push(index);
            plan(inner, output, pin, path, planned);
            path.pop();
        }
    }
}

/// `source_column` names the inner output. A caller-sized item has no output to follow.
fn projected_query<'a>(
    query: &'a SqlBoundQuery,
    index: usize,
    column: &str,
) -> Option<(&'a SqlBoundQuery, usize)> {
    let item = query.items.get(index)?;
    let SqlBoundItemKind::Query(inner) = &item.kind else {
        return None;
    };
    ordinal(inner, &item.column_aliases, column).map(|next| (inner, next))
}

fn set_operation(query: &SqlBoundQuery) -> bool {
    query.items.len() >= 2
        && query
            .items
            .iter()
            .all(|item| matches!(item.kind, SqlBoundItemKind::Query(_)))
        && query
            .outputs
            .iter()
            .all(|output| output.source_column.is_none())
}

fn apply(
    query: &mut SqlBoundQuery,
    alternatives: usize,
    planned: &BTreeMap<Vec<usize>, Vec<(String, SqlBoundPin)>>,
) {
    for (path, pins) in planned {
        let Some(item) = item_at(query, path) else {
            continue;
        };
        if !matches!(item.kind, SqlBoundItemKind::Table(_)) || item.pin_alternatives > 1 {
            continue;
        }
        if alternatives > 1 {
            item.pin_alternatives = alternatives;
        }
        for (column, pin) in pins {
            let mut copy = pin.clone();
            copy.column = column.clone();
            copy.unqualified_candidate = false;
            if !item.pins.contains(&copy) {
                item.pins.push(copy);
            }
        }
    }
}

fn item_at<'a>(query: &'a mut SqlBoundQuery, path: &[usize]) -> Option<&'a mut SqlBoundItem> {
    let (&index, rest) = path.split_first()?;
    let item = query.items.get_mut(index)?;
    if rest.is_empty() {
        return Some(item);
    }
    match &mut item.kind {
        SqlBoundItemKind::Query(inner) => item_at(inner, rest),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
