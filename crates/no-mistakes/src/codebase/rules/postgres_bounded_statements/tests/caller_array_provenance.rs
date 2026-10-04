#[test]
fn caller_array_provenance_rejects_stored_or_unknown_results() {
    let root = super::fixture_root();
    let sources =
        crate::codebase::rules::source_store_for_files(&[root.join("schema-finite-array.json")]);
    let catalog =
        crate::codebase::postgres::SchemaCatalog::load(&root, "schema-finite-array.json", &sources)
            .unwrap();
    let sql = std::fs::read_to_string(root.join("sql/caller-array-provenance.sql")).unwrap();
    let facts = crate::codebase::postgres::extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed);
    let findings: Vec<_> = facts
        .bounds
        .iter()
        .flat_map(|fact| super::offenders(fact, &catalog))
        .map(|f| (f.table, f.line))
        .collect();
    let mut expected: Vec<_> = [2, 3, 8, 10, 11, 13, 14]
        .map(|line| ("accounts".into(), line))
        .into();
    expected.extend([("orders".into(), 15), ("accounts".into(), 15)]);
    expected.extend([19, 20, 21, 22, 24].map(|line| ("accounts".into(), line)));
    assert_eq!(findings, expected);
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
