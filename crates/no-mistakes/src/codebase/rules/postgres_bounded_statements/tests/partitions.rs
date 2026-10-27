use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog};

#[test]
fn temporary_partitions_only_expose_permanent_names_after_cascade() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/temporary-partitions.sql"),
    )
    .unwrap();
    assert!(extract_sql_statement_facts(&sql).parse_failed);
}

#[test]
fn partition_ddl_uses_catalog_evidence_for_the_parent_and_child() {
    let root = crate::test_support::rule_fixture_root("postgres-bounded-statements");
    let catalog = SchemaCatalog::from_json(
        &std::fs::read_to_string(root.join("schema-partition-search-path-evidence.json")).unwrap(),
    )
    .unwrap();
    for (fixture, expected_len) in [
        ("temporary-partition-conditional-empty.sql", 3),
        ("temporary-partition-conditional-physical.sql", 2),
        ("temporary-partition-conditional-qualified.sql", 3),
    ] {
        let sql = std::fs::read_to_string(root.join("sql").join(fixture)).unwrap();
        let facts = extract_sql_statement_facts(&sql);
        let projected = crate::codebase::postgres::project_sql_bounds(&facts, &catalog);
        assert_eq!(projected.len(), expected_len, "{fixture}");
    }

    // This cross-persistence ATTACH is rejected by PostgreSQL. Its temporary
    // child still selects per-catalog replay, but the physical parent owns nothing.
    let sql =
        std::fs::read_to_string(root.join("sql/temporary-partition-conditional-rejected.sql"))
            .unwrap();
    let facts = extract_sql_statement_facts(&sql);
    assert!(facts.lifecycle.is_some());
    let projected = crate::codebase::postgres::project_sql_bounds(&facts, &catalog);
    assert_eq!(projected.len(), 1);
}
