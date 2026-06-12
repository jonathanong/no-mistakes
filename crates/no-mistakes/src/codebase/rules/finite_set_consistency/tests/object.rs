use super::extract::{extract_ts_const_object_keys, extract_ts_const_object_property};
use std::collections::BTreeSet;

#[test]
fn object_property_extraction_handles_nested_quoted_braces() {
    let source = r#"
const ROUTE_META = {
  users: { label: "brace \" }", slug: "users" },
  billing: { slug: 'billing' },
};
"#;

    assert_eq!(
        extract_ts_const_object_property(source, "ROUTE_META", "slug"),
        BTreeSet::from(["billing".to_string(), "users".to_string()])
    );
}

#[test]
fn object_property_extraction_handles_escaped_quote_literals() {
    let source = r#"
const ROUTE_META = {
  double: { slug: "a\"b" },
  single: { slug: 'can\'t' },
};
"#;

    assert_eq!(
        extract_ts_const_object_property(source, "ROUTE_META", "slug"),
        BTreeSet::from(["a\"b".to_string(), "can't".to_string()])
    );
}

#[test]
fn object_key_extraction_handles_escaped_quote_literals() {
    let source = r#"
const ROUTE_META = {
  "a\"b": { slug: "quoted" },
  'can\'t': { slug: "single" },
};
"#;

    assert_eq!(
        extract_ts_const_object_keys(source, "ROUTE_META"),
        BTreeSet::from(["a\"b".to_string(), "can't".to_string()])
    );
}

#[test]
fn object_extraction_ignores_equals_in_type_annotations() {
    let source = r#"
const ROUTE_META: Record<string, () => { slug: string }> = {
  users: { slug: "users" },
};
"#;

    assert_eq!(
        extract_ts_const_object_keys(source, "ROUTE_META"),
        BTreeSet::from(["users".to_string()])
    );
}

#[test]
fn object_key_extraction_uses_only_top_level_keys() {
    let source = r#"
const ROUTE_META = {
  users: { slug: "users" },
};
"#;

    assert_eq!(
        extract_ts_const_object_keys(source, "ROUTE_META"),
        BTreeSet::from(["users".to_string()])
    );
}

#[test]
fn object_extraction_ignores_commented_matching_declarations() {
    let source = r#"
// const ROUTE_META = { legacy: { slug: "legacy" } };
const ROUTE_META = {
  users: { slug: "users" },
};
"#;

    assert_eq!(
        extract_ts_const_object_keys(source, "ROUTE_META"),
        BTreeSet::from(["users".to_string()])
    );
}

#[test]
fn object_extraction_skips_computed_keys() {
    let source = r#"
const ROUTE_META = {
  [ROUTES.users]: { slug: "users" },
  billing: { slug: "billing" },
};
"#;

    assert_eq!(
        extract_ts_const_object_keys(source, "ROUTE_META"),
        BTreeSet::from(["billing".to_string()])
    );
}

#[test]
fn object_extraction_ignores_comment_braces_when_matching_body() {
    let source = r#"
const ROUTE_META = {
  // } old route map shape
  users: { slug: "users" },
  /* } block comment */
  billing: { slug: "billing" },
};
"#;

    assert_eq!(
        extract_ts_const_object_keys(source, "ROUTE_META"),
        BTreeSet::from(["billing".to_string(), "users".to_string()])
    );
}

#[test]
fn object_extraction_ignores_regex_literal_braces() {
    let source = r#"
const ROUTE_META = {
  users: { pattern: /}/, slug: "users" },
  billing: { pattern: /[{}]/, slug: "billing" },
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
fn object_extraction_handles_type_annotations_and_comments() {
    let source = r#"
export const ROUTE_META: Record<string, { slug: string }> = {
  // user route
  users: { slug: "users" },
  /* billing route */
  billing: { slug: "billing" },
};
"#;

    assert_eq!(
        extract_ts_const_object_keys(source, "ROUTE_META"),
        BTreeSet::from(["billing".to_string(), "users".to_string()])
    );
}

#[test]
fn object_extraction_handles_quoted_keys_unterminated_comments_and_final_values() {
    let source = r#"
const ROUTE_META = {
  "quoted-route": { slug: "quoted-route" },
  final: { slug: "final" }
};
"#;

    assert_eq!(
        extract_ts_const_object_keys(source, "ROUTE_META"),
        BTreeSet::from(["final".to_string(), "quoted-route".to_string()])
    );
    assert_eq!(
        extract_ts_const_object_property(source, "ROUTE_META", "slug"),
        BTreeSet::from(["final".to_string(), "quoted-route".to_string()])
    );
    assert!(
        extract_ts_const_object_keys("const ROUTE_META = { invalidEntry }", "ROUTE_META")
            .is_empty()
    );
    assert!(extract_ts_const_object_keys(
        "const ROUTE_META = { /* intentionally unterminated\n}",
        "ROUTE_META"
    )
    .is_empty());
}
