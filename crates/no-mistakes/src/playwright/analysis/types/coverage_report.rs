use super::{CoverageFetch, CoverageRoute, CoverageSelector, DuplicateSelector, Summary};
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageReport {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    pub summary: Summary,
    pub routes: Vec<CoverageRoute>,
    pub selectors: Vec<CoverageSelector>,
    pub duplicate_selectors: Vec<DuplicateSelector>,
    pub fetch_apis: Vec<CoverageFetch>,
}

pub(super) fn is_arc_empty<T>(arc: &Arc<Vec<T>>) -> bool {
    // Serde calls this predicate with `&Arc<Vec<T>>`, and `Arc` has no
    // `is_empty`, so this helper preserves the same skip-behavior as `Vec::is_empty`
    // for arc-backed paths.
    arc.is_empty()
}
