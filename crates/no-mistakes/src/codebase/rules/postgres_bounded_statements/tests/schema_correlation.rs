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

#[test]
fn a_qualified_reference_to_the_same_catalog_relation_is_local() {
    let sql =
        std::fs::read_to_string(fixture_root().join("sql/qualified-catalog-local.sql")).unwrap();
    assert!(names(&sql).is_empty());
    let catalog = super::catalog();
    assert!(catalog.same_relation("accounts", "accounts"));
    assert!(catalog.same_relation("public.accounts", "accounts"));
    assert!(!catalog.same_relation("audit.accounts", "accounts"));
}

#[test]
fn an_unknown_inner_relation_keeps_qualified_reads_conservative() {
    let sql =
        std::fs::read_to_string(fixture_root().join("sql/qualified-catalog-unknown.sql")).unwrap();
    assert_eq!(names(&sql), ["accounts"]);
}
