use super::super::route_index::RouteHit;
use super::Consumed;
use crate::codebase::ts_resolver::normalize_path;
use std::path::Path;

#[test]
fn a_second_targeting_lookup_is_a_cache_hit() {
    let consumed = Consumed::default();
    let file = Path::new("/repo/src/fn.mts");
    assert!(consumed.targeting(file).is_empty());
    assert!(consumed.targeting(file).is_empty());
    let memo = consumed.scan_memo.borrow();
    assert_eq!(memo.walks, 1);
    assert_eq!(memo.hits, 1);
}

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
