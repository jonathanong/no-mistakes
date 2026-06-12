use super::comments::{strip_comments, strip_sql_comments};
use super::extract::{
    extract_ts_const_object_keys, extract_ts_const_object_property, extract_ts_string_union,
};
use super::object::matching_brace;
use std::collections::BTreeSet;

#[test]
fn comment_strippers_preserve_quoted_comment_markers_and_newlines() {
    assert_eq!(
        strip_comments(r#"const value = "not // a comment"; // removed"#),
        r#"const value = "not // a comment"; "#
    );
    assert_eq!(
        strip_comments("const value = `not /* a comment */`; /* removed */\nnext"),
        "const value = `not /* a comment */`; \nnext"
    );
    assert_eq!(
        strip_sql_comments("SELECT 'it''s -- quoted'; -- removed\nSELECT 1"),
        "SELECT 'it''s -- quoted'; \nSELECT 1"
    );
    assert_eq!(
        strip_sql_comments("SELECT 1 /* removed\nstill removed */\nSELECT 2"),
        "SELECT 1 \n\nSELECT 2"
    );
}

#[test]
fn ts_comment_stripping_preserves_declaration_token_boundaries() {
    assert_eq!(
        extract_ts_string_union(r#"type/* generated */RouteName = "users""#, "RouteName"),
        BTreeSet::from(["users".to_string()])
    );
    assert_eq!(
        extract_ts_const_object_keys(
            r#"const/* generated */ROUTE_META = { users: { slug: "users" } }"#,
            "ROUTE_META"
        ),
        BTreeSet::from(["users".to_string()])
    );
}

#[test]
fn object_helpers_cover_comment_quotes_and_ignored_entries() {
    let source = r#"
const ROUTE_META = {
  // ignored: { slug: "ignored" },
  literal: { slug: "literal" },
  block: /* { } */ { slug: "block" },
  array: [",", { ignored: true }],
  ...buildRoutes({ nested: "value" }),
  templated: { slug: `template-${"ignored"}` },
};
"#;

    assert_eq!(
        extract_ts_const_object_keys(source, "ROUTE_META"),
        BTreeSet::from([
            "array".to_string(),
            "block".to_string(),
            "literal".to_string(),
            "templated".to_string()
        ])
    );
    assert_eq!(
        extract_ts_const_object_property(source, "ROUTE_META", "slug"),
        BTreeSet::from(["block".to_string(), "literal".to_string()])
    );
}

#[test]
fn object_helpers_cover_ignored_tail_entries_and_quoted_literals() {
    let source = r#"
const ROUTE_META = {
  'quoted-key': { slug: 'quoted' },
  ...buildRoutes({ nested: "value" }),
  // line comment before ignored tail
  /* block comment before ignored tail */
  [dynamicKey()]
};
"#;

    assert_eq!(
        extract_ts_const_object_keys(source, "ROUTE_META"),
        BTreeSet::from(["quoted-key".to_string()])
    );
    assert_eq!(
        extract_ts_const_object_property(source, "ROUTE_META", "slug"),
        BTreeSet::from(["quoted".to_string()])
    );
}

#[test]
fn object_helpers_stop_at_ignored_unclosed_tail_entries() {
    let source = r#"
const ROUTE_META = {
  literal: { slug: "literal" },
  ...buildRoutes({ nested: "value" })
};
const BROKEN = {
  literal: { slug: "literal" },
  /* unterminated
"#;

    assert_eq!(
        extract_ts_const_object_keys(source, "ROUTE_META"),
        BTreeSet::from(["literal".to_string()])
    );
    assert!(extract_ts_const_object_keys(source, "BROKEN").is_empty());
}

#[test]
fn matching_brace_reports_unclosed_objects() {
    assert_eq!(matching_brace("const x = { a: 1", 10), None);
}

#[test]
fn matching_brace_handles_comments_strings_and_escapes() {
    assert_eq!(
        matching_brace(r#"const x = { value: "not } yet", done: true };"#, 10),
        Some(43)
    );
    assert_eq!(
        matching_brace(r#"const x = { value: 'escaped \' }', done: true };"#, 10),
        Some(46)
    );
    assert_eq!(
        matching_brace("const x = { value: `template }`, done: true };", 10),
        Some(44)
    );
    assert_eq!(
        matching_brace("const x = { // } comment\n done: true };", 10),
        Some(37)
    );
    assert_eq!(
        matching_brace("const x = { /* } comment */ done: true };", 10),
        Some(39)
    );
    assert_eq!(matching_brace("const x = { value: a / b };", 10), Some(25));
    assert_eq!(matching_brace("const x = { /* unterminated", 10), None);
}
