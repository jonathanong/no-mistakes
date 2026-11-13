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

#[test]
fn region_parsed_view_definitions_reject_aliased_stars_and_global_joins() {
    use sqlparser::ast::{SelectItem, SetExpr};
    let path = crate::test_support::rule_fixture_root("postgres-bounded-statements")
        .join("view-key-regions/schema.json");
    let found = SchemaCatalog::from_json(&std::fs::read_to_string(path).unwrap()).unwrap();
    let star = found.view_query("star_alias").expect("star alias parses");
    let SetExpr::Select(star_select) = star.body.as_ref() else {
        panic!("star alias is not a select");
    };
    let SelectItem::Wildcard(options) = &star_select.projection[0] else {
        panic!("star alias projection is {:?}", star_select.projection[0]);
    };
    assert!(options.opt_alias.is_some());
    assert!(super::visible_key(&found, "star_alias").is_none());

    let qualified = found
        .view_query("qualified_star_alias")
        .expect("qualified star alias parses");
    let SetExpr::Select(qualified_select) = qualified.body.as_ref() else {
        panic!("qualified star alias is not a select");
    };
    let SelectItem::QualifiedWildcard(_, options) = &qualified_select.projection[0] else {
        panic!(
            "qualified star projection is {:?}",
            qualified_select.projection[0]
        );
    };
    assert!(options.opt_alias.is_some());
    assert!(super::visible_key(&found, "qualified_star_alias").is_none());

    let joined = found.view_query("global_join").expect("global join parses");
    let SetExpr::Select(joined_select) = joined.body.as_ref() else {
        panic!("global join is not a select");
    };
    assert!(joined_select.from[0].joins[0].global);
    assert!(super::visible_key(&found, "global_join").is_none());
}
