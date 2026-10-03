use crate::codebase::postgres::{extract_sql_statement_facts, SqlLimitValue};
use sqlparser::ast::Statement;

#[test]
fn uppercase_hex_preserves_strict_and_recovered_caps_with_alias_controls() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-sql-shape-policy/fixture/radix-prefix-separators/sql/uppercase-hex.sql"));
    let facts = extract_sql_statement_facts(sql);
    assert!(facts.parse_failed);
    let caps: Vec<_> = facts
        .limit_uses
        .iter()
        .map(|cap| (cap.line, cap.value))
        .collect();
    assert_eq!(
        caps,
        vec![
            (2, SqlLimitValue::Literal(255)),
            (3, SqlLimitValue::Literal(255)),
            (4, SqlLimitValue::Literal(255)),
            (6, SqlLimitValue::Other),
            (9, SqlLimitValue::Other),
        ]
    );
    let statements = super::parse_postgres_sql_lenient(sql);
    let rendered: Vec<_> = statements
        .iter()
        .filter_map(|statement| match statement {
            Statement::Query(query) => Some(query.to_string()),
            _ => None,
        })
        .collect();
    assert!(rendered.iter().any(|sql| sql == "SELECT 255"));
    assert!(rendered.iter().any(|sql| sql == "SELECT 0 AS XFF"));
    assert!(rendered.iter().any(|sql| sql == "SELECT 0 AS \"XFF\""));
}

#[test]
fn malformed_uppercase_radix_does_not_become_numeric() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-sql-shape-policy/fixture/radix-prefix-separators/sql/uppercase-hex-invalid.sql"));
    for sql in sql.lines() {
        assert!(super::parse_postgres_sql(sql).is_err(), "{sql}");
        assert!(super::parse_postgres_sql_lenient(sql).is_empty(), "{sql}");
    }
}
