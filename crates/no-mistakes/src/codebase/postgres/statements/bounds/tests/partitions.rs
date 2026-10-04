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

#[test]
fn temporary_partition_transitions_inside_do_blocks() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/temporary-partition-do.sql"),
    )
    .unwrap();
    assert!(crate::codebase::postgres::parse_postgres_sql(&sql).is_err());
    assert_eq!(
        crate::codebase::postgres::parse::parse_postgres_sql_lenient(&sql).len(),
        10
    );
    assert_eq!(facts(&sql).len(), 2);
    assert_eq!(shape(&sql), ["select: opaque", "select: orders"]);
}

#[test]
fn if_not_exists_keeps_a_standalone_temporary_child() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/temporary-partition-if-not-exists.sql"),
    )
    .unwrap();
    assert_eq!(
        crate::codebase::postgres::parse::parse_postgres_sql_lenient(&sql).len(),
        6
    );
    assert_eq!(facts(&sql).len(), 1);
    assert_eq!(shape(&sql), ["select: opaque"]);
}

#[test]
fn missing_qualified_parent_cannot_detach_temporary_child() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/temporary-partition-missing-parent.sql"),
    )
    .unwrap();
    assert_eq!(
        crate::codebase::postgres::parse::parse_postgres_sql_lenient(&sql).len(),
        6
    );
    assert_eq!(facts(&sql).len(), 1);
    assert_eq!(shape(&sql), ["select: orders"]);
}

#[test]
fn non_relation_partition_syntax_cannot_detach_temporary_child() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/temporary-partition-invalid-name.sql"),
    )
    .unwrap();
    assert_eq!(
        crate::codebase::postgres::parse::parse_postgres_sql_lenient(&sql).len(),
        8
    );
    assert_eq!(facts(&sql).len(), 1);
    assert_eq!(shape(&sql), ["select: orders"]);
}

#[test]
fn failed_reattach_preserves_original_partition_owner() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/temporary-partition-owned-reattach.sql"),
    )
    .unwrap();
    assert_eq!(facts(&sql).len(), 2);
    assert_eq!(shape(&sql), ["select: opaque", "select: child"]);
}

#[test]
fn create_temp_attaches_child_when_pg_temp_is_later_in_search_path() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/temporary-partition-created-search-path.sql"),
    )
    .unwrap();
    assert_eq!(facts(&sql).len(), 2);
    assert_eq!(shape(&sql), ["select: opaque", "select: child"]);
}

#[test]
fn failed_attach_to_ordinary_parent_keeps_standalone_child() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/temporary-partition-failed-attach.sql"),
    )
    .unwrap();
    assert_eq!(facts(&sql).len(), 3);
    assert_eq!(
        shape(&sql),
        ["select: opaque", "select: opaque", "select: child"]
    );
}

#[test]
fn failed_create_partition_of_ordinary_parent_keeps_physical_child() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/temporary-partition-failed-create.sql"),
    )
    .unwrap();
    assert_eq!(facts(&sql).len(), 1);
    assert_eq!(shape(&sql), ["select: child"]);
}

#[test]
fn failed_concurrent_detach_keeps_child_owned_in_do_and_transaction() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/temporary-partition-failed-concurrent.sql"),
    )
    .unwrap();
    assert_eq!(facts(&sql).len(), 3);
    assert_eq!(
        shape(&sql),
        ["select: child", "select: child", "select: child"]
    );
}

#[test]
fn rejected_partition_bounds_leave_standalone_temp_children_alive() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/temporary-partition-invalid-bound-lifecycle.sql"),
    )
    .unwrap();
    assert_eq!(facts(&sql).len(), 2);
    assert_eq!(shape(&sql), ["select: opaque", "select: opaque"]);
}

#[test]
fn reversed_hash_bound_attaches_temporary_child() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/temporary-partition-reversed-hash.sql"),
    )
    .unwrap();
    assert_eq!(
        crate::codebase::postgres::parse::parse_postgres_sql_lenient(&sql).len(),
        6
    );
    assert_eq!(facts(&sql).len(), 1);
    assert_eq!(shape(&sql), ["select: child"]);
}

#[test]
fn restrict_blocked_parent_drop_preserves_partition_child() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/temporary-partition-restrict-drop.sql"),
    )
    .unwrap();
    assert_eq!(facts(&sql).len(), 4);
    assert_eq!(
        shape(&sql),
        [
            "select: opaque",
            "select: opaque",
            "select: orders",
            "select: spare",
        ]
    );
}
