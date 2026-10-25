#[test]
fn with_insert_query_payload_keeps_cte_calls() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-ddl-unanalyzable/fixture/procedural/migrations/with-query-calls.sql"));
    let facts = crate::codebase::postgres::migration::extract_migration_facts(sql);
    assert!(facts.unanalyzable_statements.is_empty(), "{facts:#?}");
    // Both a CTE before the INSERT and same-line siblings must survive.
    for (line, expected) in [(6, 1), (18, 1), (28, 4), (29, 4)] {
        assert_eq!(
            facts
                .function_calls
                .iter()
                .filter(|call| call.name_parts == ["abs"] && call.line == line)
                .count(),
            expected,
            "line {line}: {facts:#?}"
        );
    }
}
