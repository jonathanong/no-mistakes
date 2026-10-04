#[test]
fn table_only_arms_keep_temporary_and_permanent_names_separate() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-table-only.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    // sqlparser cannot strictly parse successive TABLE arms in one source.
    assert!(facts.parse_failed);
    assert_eq!(facts.bounds.len(), 3);
    let catalog = super::catalog();
    let found: Vec<_> = facts
        .bounds
        .iter()
        .flat_map(|fact| super::offenders(fact, &catalog))
        .map(|finding| finding.table)
        .collect();
    assert_eq!(found, ["accounts"]);
}

#[test]
fn a_strict_table_only_arm_shadows_the_permanent_catalog() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-table-only-single.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 1);
    let catalog = super::catalog();
    assert!(facts
        .bounds
        .iter()
        .flat_map(|fact| super::offenders(fact, &catalog))
        .next()
        .is_none());
}
