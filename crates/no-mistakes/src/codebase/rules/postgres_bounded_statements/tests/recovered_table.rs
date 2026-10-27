fn bounds_len(sql: &str) -> usize {
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(facts.parse_failed);
    facts.bounds.len()
}

#[test]
fn quoted_table_in_recovered_do_body_shadows_catalog() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-table-quoted-do.sql"
    ));
    assert_eq!(bounds_len(sql), 1);
}

#[test]
fn unquoted_table_in_recovered_do_body_reads_catalog() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-table-unquoted-do.sql"
    ));
    assert_eq!(bounds_len(sql), 1);
}

#[test]
fn quoted_table_in_reconstructed_sql_shadows_catalog() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-table-quoted-reconstructed.sql"
    ));
    assert_eq!(bounds_len(sql), 1);
}
