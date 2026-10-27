#[test]
fn scalar_case_boundaries_preserve_row_ownership_and_standalone_unknown_calls() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/scalar-case-projection.sql"));
    assert!(crate::codebase::postgres::parse_postgres_sql(sql).is_ok());
}
