use super::super::{query::modifying_statement, Scope};
use sqlparser::{ast::Statement, dialect::PostgreSqlDialect, parser::Parser};

fn query(sql: &str) -> sqlparser::ast::Query {
    let mut statements = Parser::parse_sql(&PostgreSqlDialect {}, sql).unwrap();
    let Statement::Query(query) = statements.remove(0) else {
        panic!("expected a query")
    };
    *query
}

#[test]
fn merge_ctes_are_treated_as_opaque_modified_row_sources() {
    let query = query(
        "WITH changed AS (MERGE INTO target USING source ON target.id = source.id \
         WHEN MATCHED THEN DELETE) SELECT * FROM changed",
    );
    let cte = &query.with.as_ref().unwrap().cte_tables[0];
    assert!(matches!(
        modifying_statement(&cte.query),
        Some(Statement::Merge(_))
    ));
}

#[test]
fn a_parenthesized_blocking_arm_is_walked_before_an_outer_union_all_limit() {
    let query = query(
        "SELECT id FROM accounts WHERE id = $1 UNION ALL \
         (SELECT id FROM accounts WHERE id = $1 EXCEPT SELECT account_id FROM orders) LIMIT 1",
    );
    let _bound = super::super::query::bound_query(&query, &Scope::default());
}
