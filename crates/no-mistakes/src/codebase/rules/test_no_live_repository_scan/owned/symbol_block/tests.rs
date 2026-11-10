use super::super::consumers::Consumed;
use super::super::route_index::RouteHit;
use super::{blocks, route};
use std::path::Path;

#[test]
fn empty_detail_does_not_match_every_export() {
    let file = Path::new("/repo/src/fn.mts");
    let hit = RouteHit::nondirect(Path::new("/repo/tests/t.mts"), "", file);
    let consumed = Consumed::with_hits(vec![hit]);
    assert!(!blocks(&consumed, file, "fn"));
    let found = route(&consumed, file, "fn").expect("earliest hit");
    assert!(found.detail.is_empty());
    assert_eq!(found.line, 8);
}
