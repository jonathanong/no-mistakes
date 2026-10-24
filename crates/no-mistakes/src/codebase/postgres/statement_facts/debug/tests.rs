#[test]
fn debug_retains_nonempty_transformed_column_facts() {
    let fixture = crate::test_support::rule_fixture_root("postgres-no-transformed-uuid-predicates")
        .join("fail.sql");
    let source = std::fs::read_to_string(fixture).unwrap();
    let facts = crate::codebase::postgres::extract_sql_statement_facts(&source);
    assert!(!facts.transformed_column_uses.is_empty());
    assert!(format!("{facts:?}").contains("transformed_column_uses: [SqlTransformedColumnUseFact"));
}
