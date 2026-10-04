use super::super::aggregate::{has_implicit_group, orders_can_expand};
use sqlparser::{
    ast::{SetExpr, Statement},
    dialect::{BigQueryDialect, DuckDbDialect, PostgreSqlDialect},
    parser::Parser,
};

fn query(sql: &str) -> sqlparser::ast::Query {
    let mut statements = Parser::parse_sql(&PostgreSqlDialect {}, sql).unwrap();
    let sqlparser::ast::Statement::Query(query) = statements.remove(0) else {
        panic!("expected a query")
    };
    *query
}

#[test]
fn implicit_group_survives_false_where_only_for_aggregate_or_having_queries() {
    for (sql, can_expand) in [
        (
            "SELECT 1 FROM orders HAVING true ORDER BY generate_series(1, 2)",
            true,
        ),
        (
            "SELECT count(*) FROM orders ORDER BY generate_series(1, 2)",
            true,
        ),
        (
            "SELECT row_number() OVER (ORDER BY count(*)) FROM orders WHERE false ORDER BY generate_series(1, 2)",
            true,
        ),
        (
            "SELECT row_number() OVER w FROM orders WHERE false WINDOW w AS (ORDER BY count(*)) ORDER BY generate_series(1, 2)",
            true,
        ),
        (
            "SELECT row_number() OVER w2 FROM orders WHERE false WINDOW w AS (ORDER BY count(*)), w2 AS (w) ORDER BY generate_series(1, 2)",
            true,
        ),
        ("SELECT 1 FROM orders ORDER BY count(*)", false),
    ] {
        let query = query(sql);
        let SetExpr::Select(select) = &*query.body else {
            panic!("expected SELECT")
        };
        assert!(has_implicit_group(&query, select), "{sql}");
        assert_eq!(orders_can_expand(&query), can_expand, "{sql}");
    }

    for sql in [
        "SELECT 1 FROM orders",
        "SELECT 1 FROM orders ORDER BY generate_series(1, 2)",
        "SELECT 1 FROM orders GROUP BY account_id ORDER BY generate_series(1, 2)",
    ] {
        let query = query(sql);
        let SetExpr::Select(select) = &*query.body else {
            panic!("expected SELECT")
        };
        assert!(!has_implicit_group(&query, select), "{sql}");
    }

    let mut statements =
        Parser::parse_sql(&DuckDbDialect {}, "SELECT 1 FROM orders ORDER BY ALL").unwrap();
    let Statement::Query(query) = statements.remove(0) else {
        panic!("expected a query")
    };
    let SetExpr::Select(select) = &*query.body else {
        panic!("expected SELECT")
    };
    assert!(!has_implicit_group(&query, select));
    assert!(!orders_can_expand(&query));

    let mut statements = Parser::parse_sql(
        &BigQueryDialect {},
        "SELECT row_number() OVER w2 FROM orders WHERE false WINDOW w AS (ORDER BY id), w2 AS w ORDER BY generate_series(1, 2)",
    )
    .unwrap();
    let Statement::Query(query) = statements.remove(0) else {
        panic!("expected SELECT")
    };
    let SetExpr::Select(select) = &*query.body else {
        panic!("expected SELECT")
    };
    assert!(!has_implicit_group(&query, select));
}
