use super::arguments;
use sqlparser::ast::{
    Expr, Function, FunctionArguments, Ident, ObjectNamePart, ObjectNamePartFunction, Visit,
    Visitor,
};
use std::ops::ControlFlow;

#[test]
fn malformed_call_names_and_missing_arguments_do_not_invent_scalar_proof() {
    struct First(Option<Function>);
    impl Visitor for First {
        type Break = ();
        fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
            if let Expr::Function(function) = expr {
                if function.name.to_string() == "pg_catalog.lower" {
                    self.0 = Some(function.clone());
                    return ControlFlow::Break(());
                }
            }
            ControlFlow::Continue(())
        }
    }
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/scalar-call-array.sql"
    ));
    let statements =
        sqlparser::parser::Parser::parse_sql(&sqlparser::dialect::PostgreSqlDialect {}, sql)
            .unwrap();
    let mut first = First(None);
    let _ = statements.visit(&mut first);
    let original = first.0.unwrap();
    let mut function = original.clone();
    function.args = FunctionArguments::None;
    assert!(arguments(&function).is_none());
    function = original;
    // Public AST inputs may contain names from other dialects; they are not builtin identities.
    function.name.0[0] = ObjectNamePart::Function(ObjectNamePartFunction {
        name: Ident::new("identifier"),
        args: vec![],
    });
    assert!(arguments(&function).is_none());
}
