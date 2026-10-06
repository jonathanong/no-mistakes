use super::*;
use sqlparser::ast::{visit_expressions_mut, Ident, ObjectNamePart, ObjectNamePartFunction};

#[test]
fn foreign_dynamic_ast_names_do_not_become_static_function_facts() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-shape-policy/fixture/banned-functions/names.sql"
    ));
    let mut statements = crate::codebase::postgres::parse::parse_postgres_sql(sql).unwrap();
    // Public ASTs can contain another dialect's computed identifier. Preserve
    // that uncertainty instead of claiming it names a configured static call.
    let _: ControlFlow<()> = visit_expressions_mut(&mut statements, |expression| {
        if let Expr::Function(function) = expression {
            function.name.0[0] = ObjectNamePart::Function(ObjectNamePartFunction {
                name: Ident::new("identifier"),
                args: vec![],
            });
        }
        ControlFlow::Continue(())
    });
    let mut calls = Vec::new();
    for statement in &statements {
        collect(statement, &mut calls);
    }
    assert!(calls.is_empty());
}
