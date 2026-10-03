use sqlparser::{dialect::PostgreSqlDialect, tokenizer::Tokenizer};

#[test]
fn radix_repair_rejects_malformed_numbers_and_preserves_token_boundaries() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-sql-shape-policy/fixture/radix-prefix-separators/sql/token-boundaries.sql"));
    for (line, sql) in sql.lines().enumerate() {
        let tokens = Tokenizer::new(&PostgreSqlDialect {}, sql)
            .tokenize_with_location()
            .unwrap();
        let repaired = super::repair(&tokens);
        assert_eq!(repaired.is_some(), line < 2, "{sql}");
        if let Some(repaired) = repaired {
            assert_eq!(
                repaired.last().unwrap().span.start,
                tokens[tokens.len() - 2].span.start
            );
            assert_eq!(
                repaired.last().unwrap().span.end,
                tokens.last().unwrap().span.end
            );
        }
    }
}

#[test]
fn empty_hex_payload_does_not_prove_a_numeric_value() {
    // The tokenizer accepts an empty numeric-hex payload; it must not become a zero cap.
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-sql-shape-policy/fixture/radix-prefix-separators/sql/empty-hex.sql"));
    let tokens = Tokenizer::new(&PostgreSqlDialect {}, sql)
        .tokenize_with_location()
        .unwrap();
    let hex = tokens
        .iter()
        .find(|token| {
            matches!(
                token.token,
                sqlparser::tokenizer::Token::HexStringLiteral(_)
            )
        })
        .unwrap();
    assert!(super::numeric_hex(hex).is_none());
}
