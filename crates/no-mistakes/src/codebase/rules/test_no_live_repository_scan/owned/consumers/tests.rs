use super::super::route_index::RouteHit;
use super::Consumed;
use crate::codebase::ts_resolver::normalize_path;
use std::path::Path;

impl Consumed {
    pub(in crate::codebase::rules::test_no_live_repository_scan) fn containing(
        file: &Path,
    ) -> Self {
        let mut consumed = Self::default();
        consumed.files.insert(normalize_path(file));
        consumed
    }

    pub(in crate::codebase::rules::test_no_live_repository_scan) fn with_hits(
        hits: Vec<RouteHit>,
    ) -> Self {
        Self {
            hits,
            ..Self::default()
        }
    }
}
