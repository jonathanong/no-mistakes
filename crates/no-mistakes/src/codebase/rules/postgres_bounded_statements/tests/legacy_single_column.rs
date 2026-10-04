#[test]
fn sole_legacy_columns_map_aliases_without_claiming_ambiguous_or_system_columns() {
    let root = super::fixture_root();
    let path = root.join("schema-legacy-single-column.json");
    let sources = crate::codebase::rules::source_store_for_files(&[path]);
    let catalog = crate::codebase::postgres::SchemaCatalog::load(
        &root,
        "schema-legacy-single-column.json",
        &sources,
    )
    .unwrap();
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/legacy-single-column-alias.sql"));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    // Extraction stays syntactic; only catalog evaluation maps the visible key.
    assert_eq!(facts.bounds[1].query.items[0].column_aliases, ["renamed"]);
    assert_eq!(facts.bounds[1].query.items[0].pins[0].column, "renamed");
    let found: Vec<_> = facts
        .bounds
        .iter()
        .flat_map(|fact| super::offenders(fact, &catalog))
        .map(|finding| (finding.table, finding.line))
        .collect();
    assert_eq!(
        found,
        [
            ("nullable_token".into(), 5),
            ("ambiguous_tokens".into(), 6),
            ("ambiguous_tokens".into(), 7),
            ("unkeyed_token".into(), 8),
            ("tokens".into(), 11)
        ]
    );
}
