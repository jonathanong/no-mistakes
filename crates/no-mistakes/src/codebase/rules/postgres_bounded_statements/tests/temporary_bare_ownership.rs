#[test]
fn temporary_shadow_cannot_borrow_permanent_bare_column_ownership() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-bare-ownership.sql"));
    assert!(crate::codebase::postgres::parse_postgres_sql(sql).is_ok());
    assert_eq!(
        super::unbounded(sql),
        [("orders".into(), 3), ("orders".into(), 9)]
    );
}
