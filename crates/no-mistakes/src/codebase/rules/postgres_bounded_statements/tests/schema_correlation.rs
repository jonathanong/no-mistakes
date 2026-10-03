use super::{fixture_root, names};

#[test]
fn qualified_relations_keep_their_schema_in_correlation_checks() {
    let sql = std::fs::read_to_string(fixture_root().join("sql/schema-correlation.sql")).unwrap();
    assert_eq!(names(&sql), ["accounts", "accounts"]);
}
