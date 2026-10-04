#[test]
fn finite_array_facts_preserve_source_items_and_scalar_requirements() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/finite-array.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    let pins = &facts.bounds[0].query.items[0].pins;
    assert!(
        matches!(&pins[0].source, crate::codebase::postgres::SqlPinSource::Array { items, scalar_columns, .. } if items == &[1] && scalar_columns == &[(1, "account_id".to_string())])
    );
    assert_eq!(
        facts.bounds[6].query.items[1].column_aliases,
        ["account_id", "id"]
    );
}
