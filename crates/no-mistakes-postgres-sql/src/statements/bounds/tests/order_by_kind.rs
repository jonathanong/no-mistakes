use super::super::aggregate::{orders_by_aggregate, orders_can_expand};
use sqlparser::ast::{OrderByKind, Statement};
use sqlparser::dialect::DuckDbDialect;
use sqlparser::parser::Parser;
use std::path::PathBuf;

#[test]
fn order_by_all_is_not_treated_as_an_expression_list() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/fixture/sql/order-by-all.sql");
    let sql = std::fs::read_to_string(fixture).unwrap();
    let mut statements = Parser::parse_sql(&DuckDbDialect {}, &sql).unwrap();
    let Statement::Query(query) = statements.remove(0) else {
        panic!("the saved fixture must parse as a query");
    };
    assert!(matches!(
        query.order_by.as_ref().map(|order| &order.kind),
        Some(OrderByKind::All(_))
    ));
    assert!(!orders_by_aggregate(&query));
    assert!(!orders_can_expand(&query));
}
