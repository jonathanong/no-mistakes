use super::super::expressions::order_prefix_matches_for_qualifiers;
use super::super::{
    canonical_order_keys, normalize_expression, order_by_ascending, order_prefix_matches,
    parse_postgres_expression, CanonicalOrderKey,
};
use sqlparser::ast::{
    Expr, Ident, ObjectName, ObjectNamePart, OrderBy, OrderByExpr, OrderByKind, OrderByOptions,
    OrderBySort,
};

fn key(expression: &str) -> CanonicalOrderKey {
    CanonicalOrderKey {
        expression: expression.to_owned(),
        ascending: true,
        nulls_first: false,
    }
}

#[test]
fn order_by_ascending_maps_sqlparser_sort_variants() {
    assert_eq!(
        order_by_ascending(&OrderByOptions {
            sort: Some(OrderBySort::Asc),
            nulls_first: None,
        }),
        Some(true)
    );
    assert_eq!(
        order_by_ascending(&OrderByOptions {
            sort: Some(OrderBySort::Desc),
            nulls_first: None,
        }),
        Some(false)
    );
    assert_eq!(
        order_by_ascending(&OrderByOptions {
            sort: None,
            nulls_first: None,
        }),
        None
    );
    assert_eq!(
        order_by_ascending(&OrderByOptions {
            sort: Some(OrderBySort::Using(ObjectName(vec![
                ObjectNamePart::Identifier(Ident::new("<")),
            ]))),
            nulls_first: None,
        }),
        None
    );
}

#[test]
fn canonical_order_keys_reject_using_operators() {
    let omitted = OrderBy {
        kind: OrderByKind::Expressions(vec![OrderByExpr {
            expr: Expr::Identifier(Ident::new("id")),
            options: OrderByOptions {
                sort: None,
                nulls_first: None,
            },
            with_fill: None,
        }]),
        interpolate: None,
    };
    assert_eq!(
        canonical_order_keys(&omitted),
        Some(vec![CanonicalOrderKey {
            expression: "id".into(),
            ascending: true,
            nulls_first: false,
        }])
    );
    let using = OrderBy {
        kind: OrderByKind::Expressions(vec![OrderByExpr {
            expr: Expr::Identifier(Ident::new("id")),
            options: OrderByOptions {
                sort: Some(OrderBySort::Using(ObjectName(vec![
                    ObjectNamePart::Identifier(Ident::new(">")),
                ]))),
                nulls_first: Some(false),
            },
            with_fill: None,
        }]),
        interpolate: None,
    };
    assert_eq!(canonical_order_keys(&using), None);
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
