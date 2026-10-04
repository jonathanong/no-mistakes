#[test]
fn wrapped_reducer_arguments_keep_owner_bounds_and_final_leaf_guards() {
    let root = super::fixture_root();
    let path = root.join("schema-finite-array.json");
    let sources = crate::codebase::rules::source_store_for_files(&[path]);
    let catalog =
        crate::codebase::postgres::SchemaCatalog::load(&root, "schema-finite-array.json", &sources)
            .unwrap();
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/wrapped-scalar-reducer-arrays.sql"));
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
        [6, 7, 8, 10, 11, 12, 13, 18, 19, 20, 21].map(|line| ("accounts".to_string(), line))
    );
}
