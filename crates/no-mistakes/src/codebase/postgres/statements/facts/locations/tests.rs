use super::finish;
use crate::codebase::postgres::SqlStatementFileFacts;

#[test]
fn patch_gap_parse_failure_without_chunk_map_overlaps() {
    let facts = SqlStatementFileFacts {
        parse_failed: true,
        ..Default::default()
    };
    let projected = finish(super::Locations::default(), &facts, &[], "", &[], &[], None);
    assert!(projected.conflict_failure_overlaps);
    assert!(projected.reliable_conflicts.is_empty());
}
