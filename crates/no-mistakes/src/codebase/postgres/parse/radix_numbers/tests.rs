use sqlparser::{dialect::PostgreSqlDialect, tokenizer::Tokenizer};

#[test]
fn radix_repair_rejects_malformed_numbers_and_preserves_token_boundaries() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-sql-shape-policy/fixture/radix-prefix-separators/sql/token-boundaries.sql"));
    for (line, sql) in sql.lines().enumerate() {
        let tokens = Tokenizer::new(&PostgreSqlDialect {}, sql).tokenize_with_location();
        let Ok(tokens) = tokens else {
            assert!(line >= 2, "{sql}");
            continue;
        };
        let repaired = super::repair(&tokens);
        assert_eq!(repaired.is_some(), line < 2, "{sql}");
        if let Some(repaired) = repaired {
            let number = repaired
                .iter()
                .rev()
                .find(|token| matches!(token.token, sqlparser::tokenizer::Token::Number(_, _)))
                .unwrap();
            let original = tokens
                .iter()
                .rev()
                .find(|token| matches!(token.token, sqlparser::tokenizer::Token::Number(_, _)))
                .unwrap();
            assert_eq!(number.span, original.span, "{sql}");
        }
    }
}

#[test]
fn empty_hex_payload_does_not_prove_a_numeric_value() {
    // An empty `0x` prefix is not a zero. sqlparser 0.64 rejects it at the lexer.
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-sql-shape-policy/fixture/radix-prefix-separators/sql/empty-hex.sql"));
    assert!(Tokenizer::new(&PostgreSqlDialect {}, sql)
        .tokenize_with_location()
        .is_err());
}
