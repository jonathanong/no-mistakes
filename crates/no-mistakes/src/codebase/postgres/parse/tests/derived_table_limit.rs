use super::super::parse_postgres_sql;
use sqlparser::ast::{SetExpr, Statement, TableFactor};

fn derived_query(sql: &str) -> Box<sqlparser::ast::Query> {
    let statements = parse_postgres_sql(sql).expect("saved derived TABLE query must parse");
    let Statement::Query(query) = statements.into_iter().next().expect("query") else {
        panic!("expected outer query");
    };
    let SetExpr::Select(select) = *query.body else {
        panic!("expected outer SELECT");
    };
    let TableFactor::Derived { subquery, .. } =
        select.from.into_iter().next().expect("source").relation
    else {
        panic!("expected derived query");
    };
    subquery
}

#[test]
fn streaming_table_arm_retains_the_derived_limit() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/table-derived-streaming-limit.sql"
    ));
    assert!(derived_query(sql).limit_clause.is_some());
}

#[test]
fn parenthesized_table_arm_without_limit_still_parses() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/table-derived-no-limit.sql"
    ));
    assert!(derived_query(sql).limit_clause.is_none());
}

#[test]
fn incomplete_derived_table_arm_stays_invalid() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/table-derived-invalid.sql"
    ));
    assert!(parse_postgres_sql(sql).is_err());
}
