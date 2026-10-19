use super::prepared_conflicts;
use crate::codebase::postgres::{
    SqlConflictInsertFact, SqlConflictTarget, SqlInsertSourceShape, SqlStatementFileFacts,
};

fn with_locations(parse_failed: bool) -> SqlStatementFileFacts {
    SqlStatementFileFacts {
        parse_failed,
        variant_locations: Some(Default::default()),
        ..Default::default()
    }
}

fn conflict(table: &str) -> SqlConflictInsertFact {
    SqlConflictInsertFact {
        table: table.to_string(),
        target: SqlConflictTarget::Targetless,
        source: SqlInsertSourceShape::default(),
    }
}

#[test]
fn patch_gap_missing_locations_are_unanalyzable() {
    let facts = SqlStatementFileFacts::default();
    let prepared = prepared_conflicts(&facts);
    assert!(prepared.conflicts.is_empty());
    assert!(prepared.unanalyzable);
}

#[test]
fn patch_gap_conflict_error_hides_complete_conflicts() {
    let mut facts = with_locations(false);
    {
        let locations = facts.variant_locations.as_mut().expect("locations");
        locations.conflict_error = Some("conflict parse failed".to_string());
        locations.conflicts.push(conflict("items"));
    }
    let prepared = prepared_conflicts(&facts);
    assert!(prepared.conflicts.is_empty());
    assert!(prepared.unanalyzable);
}
