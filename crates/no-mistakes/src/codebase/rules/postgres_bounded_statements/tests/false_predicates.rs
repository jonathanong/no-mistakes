use super::{fixture_root, names};

#[test]
fn false_where_conjuncts_bound_statements_at_zero_rows() {
    let sql = std::fs::read_to_string(fixture_root().join("sql/false-predicates.sql")).unwrap();
    assert_eq!(names(&sql), ["accounts", "accounts", "accounts"]);
}
