use super::names;

#[test]
fn temporary_partitions_only_expose_permanent_names_after_cascade() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/temporary-partitions.sql"),
    )
    .unwrap();
    assert!(crate::codebase::postgres::extract_sql_statement_facts(&sql).parse_failed);
    assert_eq!(names(&sql), ["orders", "orders"]);
}
