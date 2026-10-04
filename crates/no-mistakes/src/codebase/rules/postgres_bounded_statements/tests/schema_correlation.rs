use super::{fixture_root, names};

#[test]
fn qualified_relations_keep_their_schema_in_correlation_checks() {
    let sql = std::fs::read_to_string(fixture_root().join("sql/schema-correlation.sql")).unwrap();
    assert_eq!(names(&sql), ["accounts", "accounts"]);
}

#[test]
fn partial_derived_aliases_keep_the_projection_suffix_local() {
    let sql =
        std::fs::read_to_string(fixture_root().join("sql/partial-projection-aliases.sql")).unwrap();
    assert!(names(&sql).is_empty());
    let control = std::fs::read_to_string(
        fixture_root().join("sql/partial-alias-outer-correlation-control.sql"),
    )
    .unwrap();
    assert_eq!(names(&control), ["accounts"]);
}
