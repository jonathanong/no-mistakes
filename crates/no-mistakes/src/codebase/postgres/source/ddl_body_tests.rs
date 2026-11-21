use super::body_sql;
use sqlparser::ast::{CreateFunctionBody, Statement};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

fn body(sql: &str) -> CreateFunctionBody {
    let statement = Parser::parse_sql(&PostgreSqlDialect {}, sql)
        .unwrap_or_else(|error| panic!("{sql}: {error}"))
        .remove(0);
    let Statement::CreateFunction(function) = statement else {
        panic!("{sql} did not parse as a function");
    };
    function
        .function_body
        .unwrap_or_else(|| panic!("{sql} has no body"))
}

#[test]
fn atomic_function_bodies_keep_a_decimal_statement_spelling() {
    let filled = body(
        "CREATE FUNCTION order_count() RETURNS int LANGUAGE sql BEGIN ATOMIC SELECT id FROM orders; END",
    );
    assert_eq!(body_sql(&filled), "BEGIN ATOMIC SELECT id FROM orders; END");
    let CreateFunctionBody::BeginAtomic(mut block) = filled else {
        panic!("expected BEGIN ATOMIC");
    };
    block.statements.clear();
    assert_eq!(
        body_sql(&CreateFunctionBody::BeginAtomic(block)),
        "BEGIN ATOMIC END"
    );
}
