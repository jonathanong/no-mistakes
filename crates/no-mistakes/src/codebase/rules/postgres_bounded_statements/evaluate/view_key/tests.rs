use crate::codebase::postgres::SchemaCatalog;

fn catalog() -> SchemaCatalog {
    let path = crate::test_support::rule_fixture_root("postgres-bounded-statements")
        .join("view-key-evidence/schema.json");
    SchemaCatalog::from_json(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn view_keys_follow_projection_names_and_row_preserving_joins() {
    let catalog = catalog();
    for (view, columns) in [
        ("aliased", vec!["visible_id"]),
        ("parenthesized", vec!["visible_id"]),
        ("qualified_star", vec!["id"]),
        ("composite", vec!["account", "item"]),
        ("left_literal", vec!["id"]),
        ("left_reverse", vec!["id"]),
        ("left_composite", vec!["id"]),
        ("left_extra_equality", vec!["id"]),
        ("left_outer", vec!["id"]),
    ] {
        assert!(
            catalog.view_query(view).is_some(),
            "{view}: invalid definition"
        );
        assert_eq!(
            super::visible_key(&catalog, view),
            Some(columns.into_iter().map(str::to_owned).collect()),
            "{view}"
        );
    }
}

#[test]
fn view_keys_reject_lost_columns_and_unproven_join_cardinality() {
    let catalog = catalog();
    for view in [
        "other_star",
        "composite_missing",
        "computed",
        "without_key",
        "unknown_base",
        "nested_view",
        "derived",
        "column_alias",
        "cte",
        "fetch",
        "empty_from",
        "cross",
        "having",
        "group_all",
        "left_partial",
        "left_null",
        "left_expression",
        "left_or",
        "left_using",
        "left_keyless",
        "left_missing",
        "left_view",
        "left_derived",
        "bare_join_projection",
        "wrong_qualifier",
        "right_join",
        "left_true",
        "left_other_qualifier",
    ] {
        assert!(
            catalog.view_query(view).is_some(),
            "{view}: invalid definition"
        );
        assert_eq!(super::visible_key(&catalog, view), None, "{view}");
    }
}
