#[test]
fn appended_ordinality_survives_one_ordinary_output_alias() {
    let root = super::fixture_root();
    let path = root.join("schema-finite-array.json");
    let sources = crate::codebase::rules::source_store_for_files(&[path]);
    let catalog =
        crate::codebase::postgres::SchemaCatalog::load(&root, "schema-finite-array.json", &sources)
            .unwrap();
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/ordinality-alias.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    let found: Vec<_> = facts
        .bounds
        .iter()
        .flat_map(|fact| super::offenders(fact, &catalog))
        .map(|finding| finding.line)
        .collect();
    assert_eq!(found, [5, 7, 9, 10]);
}
