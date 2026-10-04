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

#[test]
fn positional_aliases_preserve_pins_and_nested_query_facts() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/column-alias-list.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    for index in [0, 2, 3, 4] {
        let item = &facts.bounds[index].query.items[0];
        assert_eq!(item.column_aliases, ["tenant_id", "id"]);
        assert!(!item.pins.is_empty());
    }
    assert!(
        matches!(&facts.bounds[4].query.items[0].pins[0].source, crate::codebase::postgres::SqlPinSource::Query(inner) if matches!(&inner.items[0].kind, crate::codebase::postgres::SqlBoundItemKind::Table(name) if name == "orders"))
    );
}
