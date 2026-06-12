use super::extract::{extract_ts_const_object_keys, extract_ts_const_object_property};
use std::collections::BTreeSet;

#[test]
fn object_property_extraction_requires_whole_key_match() {
    let source = r#"
const ROUTE_META = {
  users: { grid: "compact", id: "users" },
};
"#;

    assert_eq!(
        extract_ts_const_object_property(source, "ROUTE_META", "id"),
        BTreeSet::from(["users".to_string()])
    );
}

#[test]
fn object_property_extraction_ignores_nested_properties_and_spreads() {
    let source = r#"
const ROUTE_META = {
  ...COMMON_ROUTES,
  users: { seo: { slug: "seo-users" }, slug: "users" },
  billing: { slug: "billing" },
};
"#;

    assert_eq!(
        extract_ts_const_object_keys(source, "ROUTE_META"),
        BTreeSet::from(["billing".to_string(), "users".to_string()])
    );
    assert_eq!(
        extract_ts_const_object_property(source, "ROUTE_META", "slug"),
        BTreeSet::from(["billing".to_string(), "users".to_string()])
    );
}

#[test]
fn object_property_extraction_ignores_terminal_spreads_and_non_literal_values() {
    assert!(
        extract_ts_const_object_keys("const ROUTE_META = { ...COMMON_ROUTES }", "ROUTE_META")
            .is_empty()
    );

    let source = r#"
const ROUTE_META = {
  users: "users",
  billing: { slug: ROUTE_BILLING },
};
"#;

    assert!(extract_ts_const_object_property(source, "ROUTE_META", "slug").is_empty());
}

#[test]
fn object_extraction_skips_spread_operands_with_nested_commas() {
    let source = r#"
const ROUTE_META = {
  ...buildRoutes("a", "b"),
  users: { slug: "users" },
};
"#;

    assert_eq!(
        extract_ts_const_object_keys(source, "ROUTE_META"),
        BTreeSet::from(["users".to_string()])
    );
}
