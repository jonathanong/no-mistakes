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
use crate::fx::FxHashMap;
use std::ops::Range;

/// `caller` with every hole replaced by each combination of its alternatives.
/// A hole that appears twice takes the same alternative both times. Sites that
/// share a group take the same alternative index instead of a product.
pub(super) fn fill_all(
    caller: &RecoveredSql,
    holes: &Holes,
    groups: &FxHashMap<usize, u32>,
) -> Option<Vec<RecoveredSql>> {
    Some(
        fill_with_boundaries(caller, holes, None, groups)?
            .into_iter()
            .map(|(sql, _)| sql)
            .collect(),
    )
}

pub(super) fn fill_with_boundaries(
    caller: &RecoveredSql,
    holes: &Holes,
    boundary: Option<usize>,
    groups: &FxHashMap<usize, u32>,
) -> Option<Vec<(RecoveredSql, Option<usize>)>> {
    let markers = sentinel::find_all(&caller.sql);
    let mut sites: Vec<usize> = markers.iter().map(|(_, site)| *site).collect();
    sites.sort_unstable();
    sites.dedup();
    let leaders = leaders(&sites, groups);
    if !aligned(&sites, &leaders, holes, groups) {
        return None;
    }
    let mut combos: Vec<Vec<usize>> = vec![Vec::new()];
    for site in &leaders {
        let count = holes.get(site)?.len();
        let mut next = Vec::with_capacity(combos.len() * count);
        for combo in &combos {
            for index in 0..count {
                let mut extended = combo.clone();
                extended.push(index);
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
    combos
        .iter()
        .map(|combo| {
            let mut parts = Vec::with_capacity(pieces.len() * 2);
            let mut pieces = pieces.iter().cloned();
            parts.extend(pieces.next());
            for ((range, site), piece) in markers.iter().zip(pieces) {
                let mut replacement = chosen(holes, &leaders, groups, combo, *site).clone();
                replacement.invocations.inherit(
                    &caller.invocations,
                    range.start,
                    replacement.sql.len(),
                );
                parts.push(replacement);
                parts.push(piece);
            }
            Some((
                compose(parts)?,
                boundary.and_then(|boundary| {
                    filled_boundary(caller, &markers, holes, &leaders, groups, combo, boundary)
                }),
            ))
        })
        .collect()
}

fn leaders(sites: &[usize], groups: &FxHashMap<usize, u32>) -> Vec<usize> {
    let mut seen = Vec::new();
    let mut out = Vec::new();
    for site in sites {
        if let Some(group) = groups.get(site) {
            if !seen.contains(group) {
                seen.push(*group);
                out.push(*site);
            }
        } else {
            out.push(*site);
        }
    }
    out
}

fn aligned(
    sites: &[usize],
    leaders: &[usize],
    holes: &Holes,
    groups: &FxHashMap<usize, u32>,
) -> bool {
    sites.iter().all(|site| {
        let leader = leaders[leader_slot(*site, leaders, groups)];
        match (holes.get(&leader), holes.get(site)) {
            (Some(lead), Some(follow)) => !lead.is_empty() && lead.len() == follow.len(),
            _ => false,
        }
    })
}

fn leader_slot(site: usize, leaders: &[usize], groups: &FxHashMap<usize, u32>) -> usize {
    let leader = groups.get(&site).map_or(site, |group| {
        leaders
            .iter()
            .find(|candidate| groups.get(*candidate) == Some(group))
            .copied()
            .unwrap_or(site)
    });
    leaders
        .binary_search(&leader)
        .expect("every marker's site is collected")
}

fn chosen<'a>(
    holes: &'a Holes,
    leaders: &[usize],
    groups: &FxHashMap<usize, u32>,
    combo: &[usize],
    site: usize,
) -> &'a RecoveredSql {
    &holes[&site][combo[leader_slot(site, leaders, groups)]]
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
    holes: &Holes,
    leaders: &[usize],
    groups: &FxHashMap<usize, u32>,
    combo: &[usize],
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
        append(
            &mut prefix,
            &chosen(holes, leaders, groups, combo, *site).sql,
            &mut binds,
        );
        start = range.end;
    }
    append(&mut prefix, caller.sql.get(start..boundary)?, &mut binds);
    Some(publish_placeholders(prefix).len())
}

fn append(prefix: &mut String, text: &str, binds: &mut u32) {
    prefix.push_str(&renumber_placeholders(text, *binds));
    *binds += count_placeholders(text);
}
