use super::super::expressions::{
    order_prefix_matches_for_qualifiers, order_prefix_matches_skipping_pinned,
};
use super::super::{
    normalize_expression, order_prefix_matches, parse_postgres_expression, CanonicalOrderKey,
};

fn key(expression: &str) -> CanonicalOrderKey {
    CanonicalOrderKey {
        expression: expression.to_owned(),
        ascending: true,
        nulls_first: false,
    }
}

#[test]
fn expression_comparison_requires_every_order_key_attribute() {
    assert!(order_prefix_matches(
        &[key("id"), key("created_at")],
        &[key("id")],
        false
    ));
    assert!(!order_prefix_matches(
        &[key("id")],
        &[key("id"), key("created_at")],
        false
    ));
    assert!(!order_prefix_matches(&[key("slug")], &[key("id")], false));

    let mut descending = key("id");
    descending.ascending = false;
    assert!(!order_prefix_matches(&[descending], &[key("id")], false));

    let mut nulls_first = key("id");
    nulls_first.nulls_first = true;
    assert!(!order_prefix_matches(&[nulls_first], &[key("id")], false));
    assert!(order_prefix_matches(&[key("input.id")], &[key("id")], true));
}

#[test]
fn postgres_expression_parser_handles_aliases_and_rejects_non_expressions() {
    assert_eq!(
        parse_postgres_expression("id AS alias").map(|expression| expression.to_string()),
        Some("id".to_owned())
    );
    assert!(parse_postgres_expression("*").is_none());
    assert!(parse_postgres_expression("id, name").is_none());
    assert!(parse_postgres_expression("lower(").is_none());
    assert!(!order_prefix_matches_for_qualifiers(
        &[key("lower(")],
        &[key("lower(id)")],
        &["items".to_owned()],
    ));
}

#[test]
fn normalization_preserves_quoted_strings_and_escaped_quotes() {
    assert_eq!(
        normalize_expression(" 'O''Brien'  ||  \"My Column\" "),
        "'O''Brien'||\"My Column\""
    );
}

#[test]
fn redundant_boolean_parentheses_normalize_but_precedence_is_kept() {
    // The catalog parenthesizes every conjunct; written predicates usually do not.
    assert_eq!(
        normalize_expression("((deleted_at IS NULL) AND (user_id IS NOT NULL))"),
        normalize_expression("deleted_at IS NULL AND user_id IS NOT NULL")
    );
    assert_eq!(
        normalize_expression("((a) AND ((b) AND (c)))"),
        normalize_expression("(a AND b) AND c")
    );
    assert_eq!(
        normalize_expression("(a AND b) OR c"),
        normalize_expression("a AND b OR c")
    );
    // Conjunct order stays significant; logical equivalence is not guessed.
    assert_ne!(
        normalize_expression("a IS NULL AND b IS NULL"),
        normalize_expression("b IS NULL AND a IS NULL")
    );
    // Parentheses that change precedence must survive.
    assert_ne!(
        normalize_expression("(a OR b) AND c"),
        normalize_expression("a OR b AND c")
    );
    assert_ne!(
        normalize_expression("a AND (b OR c)"),
        normalize_expression("a AND b OR c")
    );
    assert_ne!(
        normalize_expression("NOT (a AND b)"),
        normalize_expression("NOT a AND b")
    );
}

#[test]
fn pinned_leading_key_columns_drop_only_a_prefix() {
    let qualifiers = ["feed_items".to_owned()];
    let keys = [key("host_id"), key("guid")];
    let host = ["host_id".to_owned()];
    assert!(order_prefix_matches_skipping_pinned(
        &[key("guid")],
        &keys,
        &qualifiers,
        &host,
    ));
    assert!(order_prefix_matches_skipping_pinned(
        &[key("guid"), key("id")],
        &keys,
        &qualifiers,
        &host,
    ));
    assert!(order_prefix_matches_skipping_pinned(
        &[key("feed_items.guid")],
        &keys,
        &qualifiers,
        &host,
    ));
    assert!(!order_prefix_matches_skipping_pinned(
        &[key("guid")],
        &keys,
        &qualifiers,
        &[],
    ));
    assert!(!order_prefix_matches_skipping_pinned(
        &[key("slot")],
        &[key("host_id"), key("guid"), key("slot")],
        &qualifiers,
        &["guid".to_owned()],
    ));
    assert!(order_prefix_matches_skipping_pinned(
        &[key("other")],
        &keys,
        &qualifiers,
        &["host_id".to_owned(), "guid".to_owned()],
    ));
    assert!(order_prefix_matches_skipping_pinned(
        &[],
        &keys,
        &qualifiers,
        &["host_id".to_owned(), "guid".to_owned()],
    ));
    let mut descending = key("guid");
    descending.ascending = false;
    descending.nulls_first = true;
    assert!(!order_prefix_matches_skipping_pinned(
        &[descending],
        &keys,
        &qualifiers,
        &host,
    ));
    let mut nulls_first = key("guid");
    nulls_first.nulls_first = true;
    assert!(!order_prefix_matches_skipping_pinned(
        &[nulls_first],
        &keys,
        &qualifiers,
        &host,
    ));
    assert!(!order_prefix_matches_skipping_pinned(
        &[key("other.guid")],
        &keys,
        &qualifiers,
        &host,
    ));
    assert!(!order_prefix_matches_skipping_pinned(
        &[key("email")],
        &[key("lower(email)")],
        &qualifiers,
        &["email".to_owned()],
    ));
    assert!(order_prefix_matches_skipping_pinned(
        &[key("lower(email)")],
        &[key("host_id"), key("lower(email)")],
        &qualifiers,
        &host,
    ));
    assert!(order_prefix_matches_skipping_pinned(
        &[key("rest")],
        &[
            CanonicalOrderKey {
                expression: "\"Order Id\"".to_owned(),
                ascending: true,
                nulls_first: false,
            },
            key("rest"),
        ],
        &qualifiers,
        &["Order Id".to_owned()],
    ));
}
