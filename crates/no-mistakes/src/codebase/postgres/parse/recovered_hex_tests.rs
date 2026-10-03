use crate::codebase::postgres::{extract_sql_statement_facts, SqlLimitValue};

#[test]
fn reconstructed_sql_retains_numeric_hex_identity_and_source_lines() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-sql-shape-policy/fixture/radix-prefix-separators/sql/recovered-hex.sql"));
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
            (4, SqlLimitValue::Other),
            (5, SqlLimitValue::Literal(16)),
            (6, SqlLimitValue::Other),
            (7, SqlLimitValue::Other),
        ]
    );
}
