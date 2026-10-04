#[test]
fn scalar_reducers_keep_argument_row_bounds_without_scalar_argument_types() {
    let root = super::fixture_root();
    let path = root.join("schema-finite-array.json");
    let sources = crate::codebase::rules::source_store_for_files(&[path]);
    let catalog =
        crate::codebase::postgres::SchemaCatalog::load(&root, "schema-finite-array.json", &sources)
            .unwrap();
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/scalar-reducer-arrays.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    let found: Vec<_> = facts
        .bounds
        .iter()
        .flat_map(|fact| super::offenders(fact, &catalog))
        .map(|finding| (finding.table, finding.line))
        .collect();
    assert_eq!(
        found,
        [
            ("accounts", 4),
            ("accounts", 5),
            ("accounts", 6),
            ("accounts", 7),
            ("accounts", 8),
            ("accounts", 9),
            ("accounts", 11),
            ("accounts", 13),
            ("accounts", 16),
            ("accounts", 17),
            ("accounts", 18)
        ]
        .map(|(table, line)| (table.to_string(), line))
    );
}
