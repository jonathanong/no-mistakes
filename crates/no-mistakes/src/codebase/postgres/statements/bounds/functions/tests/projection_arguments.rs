use super::super::data_backed_projection;
use sqlparser::ast::{Expr, Function, FunctionArguments, Visit, Visitor};
use std::ops::ControlFlow;

#[test]
fn subquery_srf_arguments_do_not_invent_caller_sized_projection_proof() {
    struct First(Option<Function>);
    impl Visitor for First {
        type Break = ();
        fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
            if let Expr::Function(function) = expr {
                if function.name.to_string() == "generate_series" {
                    self.0 = Some(function.clone());
                    return ControlFlow::Break(());
                }
            }
            ControlFlow::Continue(())
        }
    }
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/server-state-projection.sql"));
    let statements =
        sqlparser::parser::Parser::parse_sql(&sqlparser::dialect::PostgreSqlDialect {}, sql)
            .unwrap();
    let mut first = First(None);
    let _ = statements.visit(&mut first);
    let function = first.0.unwrap();
    assert!(!data_backed_projection(&function));
    // The typed helper accepts an AST argument shape produced by other dialects;
    // a query argument supplies no caller-sized proof even if PostgreSQL rejects this spelling.
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/projection-subquery-argument.sql"));
    let statements =
        sqlparser::parser::Parser::parse_sql(&sqlparser::dialect::SnowflakeDialect {}, sql)
            .unwrap();
    let mut first = First(None);
    let _ = statements.visit(&mut first);
    let function = first.0.unwrap();
    assert!(matches!(function.args, FunctionArguments::Subquery(_)));
    assert!(data_backed_projection(&function));
}
