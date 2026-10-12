use super::*;
use sqlparser::{dialect::PostgreSqlDialect, parser::Parser};

#[test]
fn quantified_operand_recursion_has_an_independent_depth_bound() {
    let sql = crate::codebase::postgres::source::tests::fixture("query-scalar-quantified.sql");
    let statements = Parser::parse_sql(&PostgreSqlDialect {}, &sql).unwrap();
    let sqlparser::ast::Statement::Query(query) = &statements[1] else {
        panic!()
    };
    let sqlparser::ast::SetExpr::Select(select) = query.body.as_ref() else {
        panic!()
    };
    let Expr::AllOp { left, right, .. } = select.selection.as_ref().unwrap() else {
        panic!()
    };
    assert!(operand(left, false, 63));
    assert!(!operand(left, false, 64));
    assert!(operand(right, true, 63));
    assert!(!operand(right, true, 64));
}
