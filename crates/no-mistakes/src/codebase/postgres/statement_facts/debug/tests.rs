#[test]
fn debug_retains_nonempty_transformed_column_facts() {
    let fixture = crate::test_support::rule_fixture_root("postgres-no-transformed-uuid-predicates")
        .join("fail.sql");
    let source = std::fs::read_to_string(fixture).unwrap();
    let facts = crate::codebase::postgres::extract_sql_statement_facts(&source);
    assert!(!facts.transformed_column_uses.is_empty());
    assert!(format!("{facts:?}").contains("transformed_column_uses: [SqlTransformedColumnUseFact"));
}

#[test]
fn postgres_required_predicates_probe_positions_preserve_legacy_debug_bytes() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-required-predicates/fixture/index-probes/constant.sql"
    ));
    let mut facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert_eq!(facts.selects[0].sql_position, (1, 1));
    let original = format!("{facts:?}");
    for select in &mut facts.selects {
        select.sql_position = (0, 0);
    }
    assert_eq!(format!("{facts:?}"), original);
    assert!(!original.contains("sql_position"));
}
