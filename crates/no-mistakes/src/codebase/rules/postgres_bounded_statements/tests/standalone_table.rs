#[test]
fn standalone_table_queries_keep_temporary_identity_and_report_the_catalog_namesake() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/standalone-table-temporary.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 6);
    let catalog = super::catalog();
    let found: Vec<_> = facts
        .bounds
        .iter()
        .flat_map(|fact| super::offenders(fact, &catalog))
        .map(|finding| (finding.table, finding.line))
        .collect();
    assert_eq!(
        found,
        [("accounts".to_string(), 4), ("accounts".to_string(), 6)]
    );
}

#[test]
fn lenient_recovery_keeps_standalone_table_queries_after_unsupported_ddl() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/standalone-table-lenient.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(facts.parse_failed);
    assert_eq!(facts.bounds.len(), 2);
    let catalog = super::catalog();
    let found: Vec<_> = facts
        .bounds
        .iter()
        .flat_map(|fact| super::offenders(fact, &catalog))
        .map(|finding| (finding.table, finding.line))
        .collect();
    assert_eq!(found, [("accounts".to_string(), 5)]);
}
