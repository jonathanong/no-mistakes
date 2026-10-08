use super::roots::root;
use crate::codebase::postgres::source::tests::fixture;
use sqlparser::{ast::*, dialect::PostgreSqlDialect, parser::Parser};
use std::ops::ControlFlow;

fn statements(name: &str) -> Vec<Statement> {
    Parser::parse_sql(&PostgreSqlDialect {}, &fixture(name)).unwrap()
}

#[test]
fn recursive_child_roots_cover_saved_syntax_and_defensive_modifiers() {
    struct Roots(Vec<Expr>);
    impl Visitor for Roots {
        type Break = ();
        fn pre_visit_expr(&mut self, expression: &Expr) -> ControlFlow<()> {
            self.0.push(expression.clone());
            ControlFlow::Continue(())
        }
    }
    let mut expressions = Roots(Vec::new());
    let _ = statements("insert-recursive-expressions.sql").visit(&mut expressions);
    let _ = statements("insert-column-sources.sql").visit(&mut expressions);
    let tags = expressions
        .0
        .iter()
        .map(|expr| {
            serde_json::to_value(root(expr)).unwrap()["kind"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect::<std::collections::BTreeSet<_>>();
    for tag in [
        "columnReference",
        "functionCall",
        "literal",
        "binary",
        "unary",
        "parenthesized",
        "cast",
        "case",
    ] {
        assert!(tags.contains(tag), "missing {tag}");
    }
    let literal = expressions
        .0
        .iter()
        .find(|expr| matches!(expr, Expr::Value(_)))
        .unwrap()
        .clone();
    let Statement::Insert(insert) = statements("insert-column-sources.sql").remove(0) else {
        panic!()
    };
    let query = insert.source.unwrap();
    for expr in [
        Expr::Subquery(query.clone()),
        Expr::Exists {
            subquery: query.clone(),
            negated: false,
        },
        Expr::Tuple(vec![literal.clone()]),
    ] {
        let value = serde_json::to_value(root(&expr)).unwrap();
        assert!(matches!(value["kind"].as_str(), Some("subquery" | "other")));
    }
    let mut function = expressions
        .0
        .iter()
        .find_map(|expr| {
            if let Expr::Function(function) = expr {
                Some(function.clone())
            } else {
                None
            }
        })
        .unwrap();
    function.args = FunctionArguments::Subquery(query);
    assert_eq!(
        serde_json::to_value(root(&Expr::Function(function.clone()))).unwrap()["argumentsComplete"],
        false
    );
    // These AST-only modifiers exercise fail-closed grammar boundaries without inventing SQL fixtures.
    let ident = Ident::new("saved_argument");
    function.within_group = vec![OrderByExpr::from(ident.clone())];
    function.args = FunctionArguments::List(FunctionArgumentList {
        duplicate_treatment: None,
        args: vec![FunctionArg::Named {
            name: ident,
            arg: FunctionArgExpr::QualifiedWildcard(function.name.clone()),
            operator: FunctionArgOperator::RightArrow,
        }],
        clauses: vec![FunctionArgumentClause::Limit(literal)],
    });
    let value = serde_json::to_value(root(&Expr::Function(function))).unwrap();
    assert_eq!(value["argumentsComplete"], false);
    assert_eq!(value["modifiers"].as_array().unwrap().len(), 2);
}
