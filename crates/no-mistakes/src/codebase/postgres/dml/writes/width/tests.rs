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
}
