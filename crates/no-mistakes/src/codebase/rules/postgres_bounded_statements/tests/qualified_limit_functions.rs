use super::{fixture_root, names};

#[test]
fn qualified_and_quoted_conditional_function_lookalikes_are_not_caps() {
    let sql =
        std::fs::read_to_string(fixture_root().join("sql/qualified-limit-functions.sql")).unwrap();
    assert_eq!(names(&sql), ["orders", "orders", "orders"]);
}
