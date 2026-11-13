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

fn select(sql: &str) -> sqlparser::ast::Select {
    let statement = crate::codebase::postgres::parse_postgres_sql(sql)
        .unwrap()
        .remove(0);
    let sqlparser::ast::Statement::Query(query) = statement else {
        panic!("expected a query");
    };
    let sqlparser::ast::SetExpr::Select(select) = *query.body else {
        panic!("expected a select");
    };
    *select
}

#[test]
fn region_view_key_projection_rejects_shapes_the_dialect_cannot_name() {
    use sqlparser::ast::{
        Expr, Ident, ObjectName, ObjectNamePart, ObjectNamePartFunction, SelectItem,
    };
    let found = catalog();
    assert!(super::base_key(&select("SELECT 1 AS id"), &found).is_none());
    let function = ObjectName(vec![ObjectNamePart::Function(ObjectNamePartFunction {
        name: Ident::new("bar"),
        args: Vec::new(),
    })]);
    assert!(super::qualifiers(&function, None).is_empty());
    let short = Expr::CompoundIdentifier(vec![Ident::new("only")]);
    assert!(super::qualified_column(&short, &["base".into()]).is_none());

    let mut star = select("SELECT * FROM base");
    let SelectItem::Wildcard(options) = &mut star.projection[0] else {
        panic!("expected a star");
    };
    options.opt_alias = Some(Ident::new("all"));
    assert!(super::projected(&star, &["base".into()], &["id".into()], true).is_none());

    let mut qualified = select("SELECT b.* FROM base b");
    let SelectItem::QualifiedWildcard(_, options) = &mut qualified.projection[0] else {
        panic!("expected a qualified star");
    };
    options.opt_alias = Some(Ident::new("all"));
    assert!(super::projected(&qualified, &["b".into()], &["id".into()], false).is_none());

    let mut aliased = select("SELECT id FROM base");
    let SelectItem::UnnamedExpr(expr) = aliased.projection[0].clone() else {
        panic!("expected an expression");
    };
    aliased.projection[0] = SelectItem::ExprWithAliases {
        expr,
        aliases: vec![Ident::new("a"), Ident::new("b")],
    };
    assert!(super::projected(&aliased, &["base".into()], &["id".into()], true).is_none());

    let mut join = select("SELECT b.id FROM base b LEFT JOIN related r ON r.id = b.id");
    join.from[0].joins[0].global = true;
    assert!(!super::join::preserves_rows(
        &join.from[0].joins[0],
        &[],
        &found
    ));
}
