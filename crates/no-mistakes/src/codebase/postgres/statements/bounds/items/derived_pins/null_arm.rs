//! A constant NULL arm matches no row, so that alternative needs no key.
use super::item_at;
use crate::codebase::postgres::statements::{
    SqlBoundInputMode, SqlBoundItem, SqlBoundItemKind, SqlBoundPin, SqlBoundQuery,
};

pub(super) fn note(
    query: &SqlBoundQuery,
    output: usize,
    pin: &SqlBoundPin,
    path: &[usize],
    rejections: &mut Vec<(Vec<usize>, Option<usize>)>,
) -> bool {
    if query
        .outputs
        .get(output)
        .is_some_and(|output| output.constant_null)
    {
        rejections.push((path.to_vec(), pin.alternative));
        return true;
    }
    false
}

pub(super) fn apply(
    query: &mut SqlBoundQuery,
    alternatives: usize,
    rejections: &[(Vec<usize>, Option<usize>)],
) {
    for (path, alternative) in rejections {
        if path.is_empty() {
            match alternative {
                None => query.input_mode = SqlBoundInputMode::Skipped,
                Some(alternative) => {
                    for child in &mut query.items {
                        reject(child, *alternative, alternatives);
                    }
                }
            }
            continue;
        }
        let Some(item) = item_at(query, path) else {
            continue;
        };
        match alternative {
            None => skip(item),
            Some(alternative) => reject(item, *alternative, alternatives),
        }
    }
}

fn skip(item: &mut SqlBoundItem) {
    if let SqlBoundItemKind::Query(query) = &mut item.kind {
        query.input_mode = SqlBoundInputMode::Skipped;
    }
}

fn reject(item: &mut SqlBoundItem, alternative: usize, alternatives: usize) {
    match &mut item.kind {
        SqlBoundItemKind::Table(_) => {
            item.pin_alternatives = item.pin_alternatives.max(alternatives).max(alternative + 1);
            if !item.rejected_alternatives.contains(&alternative) {
                item.rejected_alternatives.push(alternative);
            }
        }
        SqlBoundItemKind::Query(query) => {
            for child in &mut query.items {
                reject(child, alternative, alternatives);
            }
        }
        _ => {}
    }
}
