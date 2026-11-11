//! A consumed module blocks one export, not every sibling const.
use super::consumers::Consumed;
use super::route_index::RouteHit;
use crate::codebase::ts_source::module_facts::arguments::RouteKind;
use std::path::Path;

pub(super) fn blocks(consumed: &Consumed, file: &Path, symbol: &str) -> bool {
    let hits = consumed.targeting(file);
    if hits.iter().any(|hit| applies(hit, symbol)) {
        return true;
    }
    consumed.contains(file) && hits.iter().all(|hit| hit.symbol.is_some())
}

/// Prefer a non-call reference to this symbol over any other hit on its module.
pub(super) fn route<'a>(consumed: &'a Consumed, file: &Path, symbol: &str) -> Option<&'a RouteHit> {
    let hits = consumed.targeting(file);
    hits.iter()
        .copied()
        .find(|hit| hit.kind == RouteKind::NonDirect && applies(hit, symbol))
        .or_else(|| {
            hits.iter()
                .copied()
                .find(|hit| hit.kind != RouteKind::NonDirect && applies(hit, symbol))
        })
        .or_else(|| {
            hits.iter()
                .copied()
                .filter(|hit| hit.symbol.is_none() || applies(hit, symbol))
                .min_by(|left, right| (&left.file, left.start).cmp(&(&right.file, right.start)))
        })
}

fn applies(hit: &RouteHit, symbol: &str) -> bool {
    hit.symbol.as_deref().map_or_else(
        || hit.kind != RouteKind::NonDirect || matches_symbol(symbol, &hit.detail),
        |name| matches_symbol(symbol, name),
    )
}

fn matches_symbol(symbol: &str, detail: &str) -> bool {
    if detail.is_empty() {
        return false;
    }
    let short = symbol.rsplit(['.', '/']).next().unwrap_or(symbol);
    detail == symbol || detail == short
}

#[cfg(test)]
#[path = "symbol_block/tests.rs"]
mod tests;
