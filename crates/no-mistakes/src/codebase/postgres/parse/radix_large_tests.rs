#[test]
fn oversized_radix_numbers_preserve_valid_statements_and_limit_classification() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-sql-shape-policy/fixture/radix-prefix-separators/sql/large.sql"));
    let statements = super::parse_postgres_sql(sql).unwrap();
    assert_eq!(statements.len(), 3);
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert_eq!(facts.limit_uses.len(), 3);
    assert!(matches!(
        facts.limit_uses[0].value,
        crate::codebase::postgres::SqlLimitValue::Other
    ));
    assert!(matches!(
        facts.limit_uses[1].value,
        crate::codebase::postgres::SqlLimitValue::Other
    ));
    assert_eq!(
        facts.limit_uses[2].value,
        crate::codebase::postgres::SqlLimitValue::Literal(15)
    );
    assert!(statements[0].to_string().contains("0o_777"));
    assert!(statements[1].to_string().contains("0b_111"));
}

#[test]
fn overflow_does_not_hide_an_invalid_later_radix_digit() {
    // The integer decoder can report overflow before reaching the invalid final digit.
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-sql-shape-policy/fixture/radix-prefix-separators/sql/large-invalid.sql"));
    for line in sql.lines() {
        let tokens =
            sqlparser::tokenizer::Tokenizer::new(&sqlparser::dialect::PostgreSqlDialect {}, line)
                .tokenize_with_location()
                .unwrap();
        assert!(super::radix_numbers::repair(&tokens).is_none());
        assert!(super::parse_postgres_sql(line).is_err());
    }
}
