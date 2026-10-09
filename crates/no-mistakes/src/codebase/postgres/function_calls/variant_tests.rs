use super::*;
use sqlparser::ast::{
    FunctionArgumentClause, FunctionArguments, Ident, NamedWindowExpr, ObjectNamePart, OrderBy,
    OrderByKind, SelectItem, SelectItemQualifiedWildcardKind, SetExpr, WildcardAdditionalOptions,
};

fn seeds() -> Vec<Statement> {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres/function-call-clauses/variants.sql"
    ));
    crate::codebase::postgres::parse::parse_postgres_sql(source).unwrap()
}

fn select(statement: &mut Statement) -> &mut Select {
    let Statement::Query(query) = statement else {
        panic!("expected query fixture");
    };
    let SetExpr::Select(select) = query.body.as_mut() else {
        panic!("expected SELECT fixture");
    };
    select
}

#[test]
fn projected_expressions_keep_their_clause_under_alias_and_wildcard_wrappers() {
    let mut statement = seeds().remove(0);
    let SelectItem::ExprWithAlias { expr, alias } = &select(&mut statement).projection[0] else {
        panic!("expected aliased expression fixture");
    };
    let expression = expr.clone();
    let alias = alias.clone();
    let variants = [
        SelectItem::UnnamedExpr(expression.clone()),
        SelectItem::ExprWithAlias {
            expr: expression.clone(),
            alias: alias.clone(),
        },
        SelectItem::ExprWithAliases {
            expr: expression.clone(),
            aliases: vec![alias.clone(), Ident::new("second")],
        },
        SelectItem::QualifiedWildcard(
            SelectItemQualifiedWildcardKind::Expr(expression),
            WildcardAdditionalOptions::default(),
        ),
    ];
    for item in variants {
        select(&mut statement).projection = vec![item];
        let mut calls = Vec::new();
        collect(&statement, &mut calls);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name_parts, ["probe_projection"]);
        assert_eq!(calls[0].clause, Some(SqlFunctionClause::SelectList));
        assert_eq!(calls[0].line, 1);
    }
    for item in [
        SelectItem::QualifiedWildcard(
            SelectItemQualifiedWildcardKind::ObjectName(ObjectName(vec![
                ObjectNamePart::Identifier(alias),
            ])),
            WildcardAdditionalOptions::default(),
        ),
        SelectItem::Wildcard(WildcardAdditionalOptions::default()),
    ] {
        select(&mut statement).projection = vec![item];
        let mut calls = Vec::new();
        collect(&statement, &mut calls);
        assert!(
            calls.is_empty(),
            "a wildcard name is not a function expression"
        );
    }
}

#[test]
fn named_window_references_and_order_all_do_not_invent_expression_roots() {
    let mut statement = seeds().remove(1);
    // These foreign AST wrappers carry names/options, not executable expressions.
    select(&mut statement).named_window[0].1 = NamedWindowExpr::NamedWindow(Ident::new("other"));
    let Statement::Query(query) = &mut statement else {
        panic!("expected query fixture");
    };
    query.order_by = Some(OrderBy {
        kind: OrderByKind::All(Default::default()),
        interpolate: None,
    });
    let mut calls = Vec::new();
    collect(&statement, &mut calls);
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name_parts, ["sum"]);
    assert_eq!(calls[0].clause, Some(SqlFunctionClause::SelectList));
}

#[test]
fn unlisted_function_argument_clauses_inherit_the_enclosing_clause() {
    let mut statements = seeds();
    let SelectItem::ExprWithAlias { expr, .. } = &select(&mut statements[0]).projection[0] else {
        panic!("expected aliased expression fixture");
    };
    let expression = expr.clone();
    let mut statement = statements.remove(2);
    let SelectItem::UnnamedExpr(Expr::Function(function)) =
        &mut select(&mut statement).projection[0]
    else {
        panic!("expected aggregate fixture");
    };
    let FunctionArguments::List(arguments) = &mut function.args else {
        panic!("expected aggregate arguments");
    };
    arguments
        .clauses
        .push(FunctionArgumentClause::Limit(expression));
    let mut calls = Vec::new();
    collect(&statement, &mut calls);
    assert_eq!(calls.len(), 2);
    assert!(calls
        .iter()
        .all(|call| call.clause == Some(SqlFunctionClause::SelectList)));
}
