#[test]
fn sole_legacy_columns_map_aliases_without_claiming_ambiguous_or_system_columns() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/legacy-single-column-alias.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    // Extraction stays syntactic; only catalog evaluation maps the visible key.
    assert_eq!(facts.bounds[1].query.items[0].column_aliases, ["renamed"]);
    assert_eq!(facts.bounds[1].query.items[0].pins[0].column, "renamed");
}
