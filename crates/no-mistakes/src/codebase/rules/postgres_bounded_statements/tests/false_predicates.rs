use super::{fixture_root, names};

#[test]
fn false_where_conjuncts_bound_statements_at_zero_rows() {
    let sql = std::fs::read_to_string(fixture_root().join("sql/false-predicates.sql")).unwrap();
    assert_eq!(
        names(&sql),
        ["orders", "accounts", "accounts", "accounts", "orders"]
    );
}

#[test]
fn empty_grouping_sets_preserve_rows_after_a_false_where() {
    let sql = std::fs::read_to_string(fixture_root().join("sql/empty-grouping-sets.sql")).unwrap();
    assert_eq!(names(&sql), ["orders"; 8]);
}

#[test]
fn aggregates_inside_inline_and_named_windows_survive_false_where() {
    let sql = std::fs::read_to_string(fixture_root().join("sql/window-aggregate.sql")).unwrap();
    assert_eq!(names(&sql), ["orders", "orders", "orders"]);
}
