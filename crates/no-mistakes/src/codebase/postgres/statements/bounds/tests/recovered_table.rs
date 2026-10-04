use super::shape;

#[test]
fn recovered_table_arm_keeps_the_query_fact_and_quoted_identity() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-table-quoted-do.sql"
    ));
    assert_eq!(shape(sql), ["select: () (\"Accounts\")"]);
}

#[test]
fn reconstructed_table_arm_keeps_the_query_fact_and_temporary_identity() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-table-quoted-reconstructed.sql"
    ));
    assert_eq!(shape(sql), ["select: () (opaque)"]);
}

#[test]
fn recovered_source_does_not_enable_unrequested_bounds() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-table-quoted-do.sql"
    ));
    let facts =
        crate::codebase::postgres::statements::extract_sql_statement_facts_with_bounds(sql, false);
    assert!(facts.parse_failed);
    assert!(facts.bounds.is_empty());
}
