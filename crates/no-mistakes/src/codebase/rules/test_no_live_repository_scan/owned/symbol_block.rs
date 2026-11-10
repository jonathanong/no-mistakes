//! A consumed module blocks one export, not every sibling const.
use super::consumers::Consumed;
use super::route_index::RouteHit;
use crate::codebase::ts_source::module_facts::arguments::RouteKind;
use std::path::Path;

pub(super) fn blocks(consumed: &Consumed, file: &Path, symbol: &str) -> bool {
    let hits = consumed.targeting(file);
    if hits.iter().any(|hit| hit.kind != RouteKind::NonDirect) {
        return true;
    }
    if hits.iter().any(|hit| matches_symbol(symbol, &hit.detail)) {
        return true;
    }
    consumed.contains(file) && hits.is_empty()
}

/// Prefer a non-call reference to this symbol over any other hit on its module.
pub(super) fn route<'a>(consumed: &'a Consumed, file: &Path, symbol: &str) -> Option<&'a RouteHit> {
    let hits = consumed.targeting(file);
    hits.iter()
        .copied()
        .find(|hit| hit.kind == RouteKind::NonDirect && matches_symbol(symbol, &hit.detail))
        .or_else(|| {
            hits.iter()
                .copied()
                .find(|hit| hit.kind != RouteKind::NonDirect)
        })
        .or_else(|| {
            hits.iter()
                .copied()
                .min_by(|left, right| (&left.file, left.start).cmp(&(&right.file, right.start)))
        })
}

fn matches_symbol(symbol: &str, detail: &str) -> bool {
    if detail.is_empty() {
        return false;
    }
    let short = symbol.rsplit(['.', '/']).next().unwrap_or(symbol);
    detail == symbol || detail == short
}
