#[test]
fn caller_array_provenance_rejects_stored_or_unknown_results() {
    let root = super::fixture_root();
    let sql = std::fs::read_to_string(root.join("sql/caller-array-provenance.sql")).unwrap();
    let facts = crate::codebase::postgres::extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed);
    use crate::codebase::postgres::SqlPinSource;
    assert!(matches!(
        facts.bounds[0].query.items[0].pins[0].source,
        SqlPinSource::StoredArray(_)
    ));
    assert!(matches!(
        facts.bounds[2].query.items[0].pins[0].source,
        SqlPinSource::Value
    ));
    assert!(matches!(
        facts.bounds[7].query.items[0].pins[0].source,
        SqlPinSource::ReadQuery(_)
    ));
    assert!(matches!(
        facts.bounds[9].query.items[0].pins[0].source,
        SqlPinSource::Query(_)
    ));
}
