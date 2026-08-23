use super::query_value_width;
use crate::codebase::postgres::parse_postgres_sql;
use sqlparser::ast::{Query, Statement};

fn insert_source(sql: &str) -> Query {
    match parse_postgres_sql(sql).unwrap().pop().expect("stmt") {
        Statement::Insert(insert) => *insert.source.expect("source"),
        other => panic!("expected insert, got {other:?}"),
    }
}

#[test]
fn query_width_covers_values_select_union_and_unknown() {
    assert_eq!(
        query_value_width(&insert_source("INSERT INTO t VALUES (1, 2), (3)")),
        Some(2)
    );
    assert_eq!(
        query_value_width(&insert_source("INSERT INTO t SELECT 1, 2")),
        Some(2)
    );
    assert_eq!(
        query_value_width(&insert_source(
            "INSERT INTO t (SELECT 1 UNION ALL SELECT 2, 3)"
        )),
        Some(2)
    );
    let unknown = parse_postgres_sql("INSERT INTO t VALUES (1)")
        .unwrap()
        .pop()
        .expect("insert");
    let mut table_query = insert_source("INSERT INTO t SELECT 1");
    *table_query.body = sqlparser::ast::SetExpr::Insert(unknown.clone());
    assert_eq!(query_value_width(&table_query), None);
    let mut union_query = insert_source("INSERT INTO t SELECT 1 UNION ALL SELECT 2");
    if let sqlparser::ast::SetExpr::SetOperation { right, .. } = union_query.body.as_mut() {
        **right = sqlparser::ast::SetExpr::Insert(unknown);
    }
    assert_eq!(query_value_width(&union_query), Some(1));
}
