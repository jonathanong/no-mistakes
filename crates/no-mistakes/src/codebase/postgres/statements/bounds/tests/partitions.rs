use super::{facts, shape};

#[test]
fn temporary_partitions_cascade_detach_and_reattach() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/temporary-partitions.sql"),
    )
    .unwrap();
    assert!(crate::codebase::postgres::parse_postgres_sql(&sql).is_err());
    assert_eq!(
        crate::codebase::postgres::parse::parse_postgres_sql_lenient(&sql).len(),
        16
    );
    let extracted = facts(&sql);
    assert_eq!(extracted.len(), 5);
    assert_eq!(
        shape(&sql),
        [
            "select: opaque",
            "select: orders",
            "select: opaque",
            "select: opaque",
            "select: orders",
        ]
    );
}

#[test]
fn temporary_partition_owner_drop_and_concurrent_detach() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/temporary-partition-review.sql"),
    )
    .unwrap();
    assert!(crate::codebase::postgres::parse_postgres_sql(&sql).is_err());
    assert_eq!(
        crate::codebase::postgres::parse::parse_postgres_sql_lenient(&sql).len(),
        21
    );
    let extracted = facts(&sql);
    assert_eq!(extracted.len(), 4);
    assert_eq!(
        shape(&sql),
        [
            "select: orders",
            "select: orders",
            "select: renamed_orders",
            "select: opaque",
        ]
    );
}
