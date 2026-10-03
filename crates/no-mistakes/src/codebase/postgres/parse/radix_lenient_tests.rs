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

#[test]
fn malformed_migration_retains_radix_select_and_write_facts() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-sql-shape-policy/fixture/radix-prefix-separators/sql/mixed-malformed.sql"));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(facts.parse_failed);
    assert_eq!(facts.selects.len(), 2);
    assert_eq!(facts.inserts.len(), 1);
    assert_eq!(facts.inserts[0].line, 4);
    assert_eq!(facts.updates.len(), 1);
    assert_eq!(facts.updates[0][0].line, 3);
    assert_eq!(facts.updates[0][0].table, "orders");
    assert_eq!(
        facts
            .limit_uses
            .iter()
            .map(|cap| (cap.line, cap.value))
            .collect::<Vec<_>>(),
        vec![
            (1, crate::codebase::postgres::SqlLimitValue::Literal(15)),
            (5, crate::codebase::postgres::SqlLimitValue::Literal(2))
        ]
    );
}
