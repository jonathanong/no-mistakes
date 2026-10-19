use super::prepared_conflicts;
use crate::codebase::postgres::statements::SqlVariantLocations;

#[test]
fn conflict_error_on_a_clean_parse_is_unanalyzable() {
    let locations = SqlVariantLocations {
        conflict_error: Some("could not align ON CONFLICT clauses".to_string()),
        ..Default::default()
    };
    let prepared = prepared_conflicts(&locations, false);
    assert!(prepared.conflicts.is_empty());
    assert!(prepared.unanalyzable);
}
