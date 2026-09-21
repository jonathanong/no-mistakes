use super::super::{canonical_order_keys, order_by_ascending, CanonicalOrderKey};
use sqlparser::ast::{
    Expr, Ident, ObjectName, ObjectNamePart, OrderBy, OrderByExpr, OrderByKind, OrderByOptions,
    OrderBySort,
};

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
