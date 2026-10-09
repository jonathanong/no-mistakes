use super::*;
use sqlparser::tokenizer::Tokenizer;

#[test]
fn procedural_recovery_handles_malformed_prefixes_with_or_without_locations() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-shape-policy/fixture/banned-functions/recovery.sql"
    ));
    let located = Tokenizer::new(&PostgreSqlDialect {}, sql)
        .tokenize_with_location()
        .unwrap();
    for with_locations in [true, false] {
        let mut counts = Vec::new();
        let mut calls = Vec::new();
        for chunk in located
            .split(|token| token.token == Token::SemiColon)
            .take(11)
        {
            let tokens: Vec<_> = chunk.iter().map(|token| token.token.clone()).collect();
            let recovered = recover(&tokens, with_locations.then_some(chunk));
            counts.push(recovered.len());
            for (statement, synthetic) in &recovered {
                crate::codebase::postgres::function_calls::collect_projected(
                    statement, &mut calls, *synthetic,
                );
            }
        }
        assert_eq!(counts, vec![1, 2, 1, 1, 0, 0, 0, 1, 1, 2, 1]);
        assert_eq!(calls.len(), 9);
    }
}
