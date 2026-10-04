#[test]
fn conditional_table_inputs_are_caller_sized_only_without_data_or_arbitrary_calls() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/conditional-table-function-inputs.sql"));
    let parsed = crate::codebase::postgres::parse_postgres_sql(sql);
    assert!(parsed.is_ok(), "{parsed:?}");
    assert_eq!(
        super::unbounded(sql),
        [7, 8, 9, 10, 11, 12, 13].map(|line| ("accounts".to_string(), line))
    );
}
