fn findings(sql: &str) -> (usize, Vec<String>) {
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(facts.parse_failed);
    let catalog = super::catalog();
    let found = facts
        .bounds
        .iter()
        .flat_map(|fact| super::offenders(fact, &catalog))
        .map(|finding| finding.table)
        .collect();
    (facts.bounds.len(), found)
}

#[test]
fn quoted_table_in_recovered_do_body_shadows_catalog() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-table-quoted-do.sql"
    ));
    assert_eq!(findings(sql), (1, vec![]));
}

#[test]
fn unquoted_table_in_recovered_do_body_reads_catalog() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-table-unquoted-do.sql"
    ));
    assert_eq!(findings(sql), (1, vec!["accounts".to_string()]));
}

#[test]
fn quoted_table_in_reconstructed_sql_shadows_catalog() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-table-quoted-reconstructed.sql"
    ));
    assert_eq!(findings(sql), (1, vec![]));
}
