#[test]
fn retained_stored_array_pins_never_credit_a_bounded_owner() {
    let root = super::fixture_root();
    let path = root.join("schema-finite-array.json");
    let sources = crate::codebase::rules::source_store_for_files(&[path]);
    let catalog =
        crate::codebase::postgres::SchemaCatalog::load(&root, "schema-finite-array.json", &sources)
            .unwrap();
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/stored-array-facts.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    let findings: Vec<_> = facts
        .bounds
        .iter()
        .flat_map(|fact| super::offenders(fact, &catalog))
        .map(|finding| (finding.table, finding.line))
        .collect();
    assert_eq!(
        findings,
        [2, 3, 4, 10, 12].map(|line| ("accounts".to_string(), line))
    );
}

#[test]
fn wrapped_self_arrays_remain_unbounded() {
    let root = super::fixture_root();
    let sources =
        crate::codebase::rules::source_store_for_files(&[root.join("schema-finite-array.json")]);
    let catalog =
        crate::codebase::postgres::SchemaCatalog::load(&root, "schema-finite-array.json", &sources)
            .unwrap();
    let facts = crate::codebase::postgres::extract_sql_statement_facts(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/wrapped-stored-array-facts.sql")));
    assert!(!facts.parse_failed);
    let lines: Vec<_> = facts
        .bounds
        .iter()
        .flat_map(|fact| super::offenders(fact, &catalog))
        .map(|finding| finding.line)
        .collect();
    assert_eq!(lines, [2, 3, 4, 7, 8, 9]);
}
