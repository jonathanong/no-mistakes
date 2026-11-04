use super::super::{
    sentinel,
    split::{compose, split},
    unpublish::unpublish,
};
use super::{Holes, RecoveredSql};
use crate::codebase::postgres::embedded::placeholders::{
    count_placeholders, publish_placeholders, renumber_placeholders,
};
use crate::codebase::postgres::embedded::{EmbeddedSqlVariant, MAX_EMBEDDED_SQL_VARIANTS};
use std::ops::Range;

/// `caller` with every hole replaced by each combination of its alternatives.
/// A hole that appears twice takes the same alternative both times.
pub(super) fn fill_all(caller: &RecoveredSql, holes: &Holes) -> Option<Vec<RecoveredSql>> {
    Some(
        fill_with_boundaries(caller, holes, None)?
            .into_iter()
            .map(|(sql, _)| sql)
            .collect(),
    )
}

pub(super) fn fill_with_boundaries(
    caller: &RecoveredSql,
    holes: &Holes,
    boundary: Option<usize>,
) -> Option<Vec<(RecoveredSql, Option<usize>)>> {
    let markers = sentinel::find_all(&caller.sql);
    let mut sites: Vec<usize> = markers.iter().map(|(_, site)| *site).collect();
    sites.sort_unstable();
    sites.dedup();
    let mut combos: Vec<Vec<&RecoveredSql>> = vec![Vec::new()];
    for site in &sites {
        let alternatives = holes.get(site).filter(|texts| !texts.is_empty())?;
        let mut next = Vec::with_capacity(combos.len() * alternatives.len());
        for combo in &combos {
            for text in alternatives {
                let mut extended = combo.clone();
                extended.push(text);
                next.push(extended);
            }
        }
        if next.len() > MAX_EMBEDDED_SQL_VARIANTS {
            return None;
        }
        combos = next;
    }
    let ranges = markers
        .iter()
        .map(|(range, _)| range.clone())
        .collect::<Vec<_>>();
    let (pieces, _) = split(caller, &ranges);
    // Which combination slot fills each marker, in text order.
    let slots: Vec<usize> = markers
        .iter()
        .map(|(_, site)| {
            sites
                .binary_search(site)
                .expect("every marker's site is collected")
        })
        .collect();
    combos
        .iter()
        .map(|combo| {
            let mut parts = Vec::with_capacity(pieces.len() * 2);
            let mut pieces = pieces.iter().cloned();
            parts.extend(pieces.next());
            for (slot, piece) in slots.iter().zip(pieces) {
                parts.push(combo[*slot].clone());
                parts.push(piece);
            }
            Some((
                compose(parts)?,
                boundary.and_then(|boundary| {
                    filled_boundary(caller, &markers, &sites, combo, boundary)
                }),
            ))
        })
        .collect()
}

pub(super) fn internal_boundary(variant: &EmbeddedSqlVariant) -> Option<usize> {
    let at = variant.first_append_offset?;
    let prefix = variant.sql_text.get(..at)?;
    let coordinate = super::super::split::coordinate(&variant.sql_text, at);
    let positions: Vec<_> = variant
        .recovered_placeholder_positions
        .iter()
        .copied()
        .filter(|position| *position < coordinate)
        .collect();
    unpublish(prefix, &positions).map(|prefix| prefix.len())
}

fn filled_boundary(
    caller: &RecoveredSql,
    markers: &[(Range<usize>, usize)],
    sites: &[usize],
    combo: &[&RecoveredSql],
    boundary: usize,
) -> Option<usize> {
    let mut prefix = String::new();
    let (mut start, mut binds) = (0, 0);
    for (range, site) in markers {
        if range.start >= boundary {
            break;
        }
        if range.end > boundary {
            return None;
        }
        append(&mut prefix, &caller.sql[start..range.start], &mut binds);
        let slot = sites.binary_search(site).ok()?;
        append(&mut prefix, &combo[slot].sql, &mut binds);
        start = range.end;
    }
    append(&mut prefix, caller.sql.get(start..boundary)?, &mut binds);
    Some(publish_placeholders(prefix).len())
}

fn append(prefix: &mut String, text: &str, binds: &mut u32) {
    prefix.push_str(&renumber_placeholders(text, *binds));
    *binds += count_placeholders(text);
}
