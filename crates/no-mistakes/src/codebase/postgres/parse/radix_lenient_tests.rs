use sqlparser::ast::{Spanned, Statement};

#[test]
fn lenient_recovery_retains_radix_limits_and_original_lines() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-sql-shape-policy/fixture/radix-prefix-separators/sql/mixed.sql"));
    assert!(super::parse_postgres_sql(sql).is_err());
    let queries = super::parse_postgres_sql_lenient(sql);
    assert_eq!(queries.len(), 2);
    let rows: Vec<_> = queries
        .iter()
        .map(|statement| {
            let Statement::Query(query) = statement else {
                panic!("expected recovered query")
            };
            (query.span().start.line, query.to_string())
        })
        .collect();
    assert_eq!(
        rows,
        vec![
            (2, "SELECT * FROM orders LIMIT 15".into()),
            (3, "SELECT * FROM orders LIMIT 2".into())
        ]
    );
}
