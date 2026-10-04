use super::shape;

#[test]
fn explained_table_queries_keep_executed_identity() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/explain-table.sql"
    ));
    assert!(
        crate::codebase::postgres::parse_postgres_sql(sql).is_ok(),
        "{:?}",
        crate::codebase::postgres::parse_postgres_sql(sql).err()
    );
    assert_eq!(
        shape(sql),
        [
            "select: opaque",
            "select: accounts",
            "select: public.\"Accounts\""
        ]
    );
}

#[test]
fn lenient_explain_table_recovery_keeps_source_identity() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/explain-table-lenient.sql"
    ));
    assert!(super::extract_sql_statement_facts(sql).parse_failed);
    assert_eq!(shape(sql), ["select: opaque", "select: accounts"]);
}
