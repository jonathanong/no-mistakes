use super::{fixture_root, names};

#[test]
fn aliased_nested_joins_preserve_catalog_column_ownership() {
    let sql = std::fs::read_to_string(fixture_root().join("sql/aliased-nested-joins.sql")).unwrap();
    assert_eq!(names(&sql), ["orders", "orders"]);
}
