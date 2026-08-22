use super::*;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::tokenizer::Tokenizer;

fn tokens(sql: &str) -> Vec<Token> {
    Tokenizer::new(&PostgreSqlDialect {}, sql)
        .tokenize()
        .expect("tokenize")
}

#[test]
fn strips_drop_index_concurrently() {
    let mut parsed = tokens("DROP INDEX CONCURRENTLY IF EXISTS idx_history__topic_id");
    rewrite_drop_index_concurrently(&mut parsed);
    assert!(parsed
        .iter()
        .all(|token| keyword_of(token) != Some(Keyword::CONCURRENTLY)));
    assert!(parsed
        .iter()
        .any(|token| keyword_of(token) == Some(Keyword::DROP)));
}

#[test]
fn rewrites_chr_to_a_string_literal() {
    let mut parsed = tokens("ESCAPE chr(92)");
    rewrite_chr_calls(&mut parsed);
    assert!(parsed.iter().any(|token| matches!(
        token,
        Token::SingleQuotedString(value) if value == "\\"
    )));
}
