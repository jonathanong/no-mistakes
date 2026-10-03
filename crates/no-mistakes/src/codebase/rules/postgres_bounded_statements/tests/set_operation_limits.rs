use super::{fixture_root, names};

#[test]
fn outer_limits_preserve_non_streaming_set_operation_work() {
    let sql = std::fs::read_to_string(fixture_root().join("sql/set-operation-limit.sql")).unwrap();
    assert_eq!(names(&sql), ["orders", "orders", "orders", "orders"]);
}
