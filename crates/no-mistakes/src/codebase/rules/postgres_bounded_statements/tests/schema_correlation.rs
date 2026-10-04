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

#[test]
fn an_explicit_inner_alias_does_not_make_a_qualified_outer_read_local() {
    let sql =
        std::fs::read_to_string(fixture_root().join("sql/qualified-catalog-aliased-inner.sql"))
            .unwrap();
    assert_eq!(names(&sql), ["accounts"]);
}

#[test]
fn a_join_alias_hides_its_children_from_catalog_qualified_locality() {
    let sql =
        std::fs::read_to_string(fixture_root().join("sql/qualified-catalog-aliased-join.sql"))
            .unwrap();
    assert_eq!(names(&sql), ["accounts", "accounts"]);
}

#[test]
fn hidden_join_two_part_qualifier_keeps_outer_write_unbounded() {
    let sql = std::fs::read_to_string(fixture_root().join("sql/hidden-join-two-part.sql")).unwrap();
    assert_eq!(
        names(&sql),
        ["accounts", "accounts", "accounts", "accounts", "accounts", "accounts", "accounts"]
    );
}

#[test]
fn joined_aliases_do_not_shadow_earlier_internal_reads() {
    let sql = std::fs::read_to_string(fixture_root().join("sql/joined-alias-timing.sql")).unwrap();
    let expected = [true, false, true, false, true, true, true, false, true];
    let queries: Vec<_> = sql
        .lines()
        .filter(|line| line.starts_with("DELETE"))
        .collect();
    assert_eq!(queries.len(), expected.len());
    for (query, unbounded) in queries.into_iter().zip(expected) {
        assert_eq!(!names(query).is_empty(), unbounded, "{query}");
    }
}
