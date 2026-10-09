use super::literal_command;
use crate::codebase::postgres::source::tests::fixture;
use sqlparser::{
    ast::{Expr, Visit, Visitor},
    dialect::PostgreSqlDialect,
    parser::Parser,
};
use std::ops::ControlFlow;

#[test]
fn literal_command_recursion_is_bounded_even_for_parser_native_syntax() {
    struct Literal(Option<Expr>);
    impl Visitor for Literal {
        type Break = ();
        fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
            if self.0.is_none()
                && matches!(expr, Expr::Value(value) if matches!(value.value, sqlparser::ast::Value::SingleQuotedString(_)))
            {
                self.0 = Some(expr.clone());
            }
            ControlFlow::Continue(())
        }
    }
    let statements =
        Parser::parse_sql(&PostgreSqlDialect {}, &fixture("expression-roots.sql")).unwrap();
    let mut literal = Literal(None);
    let _ = statements.visit(&mut literal);
    let mut expression = literal.0.unwrap();
    // Defensive parser-AST mutation verifies the boundary without generating a fixture at runtime.
    for _ in 0..63 {
        expression = Expr::Nested(Box::new(expression));
    }
    assert!(literal_command(&expression, 0).is_some());
    expression = Expr::Nested(Box::new(expression));
    assert!(literal_command(&expression, 0).is_none());
}
