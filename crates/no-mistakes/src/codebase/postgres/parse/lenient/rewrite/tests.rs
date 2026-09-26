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

#[test]
fn strips_referential_set_column_lists_only() {
    for sql in [
        "ON DELETE SET NULL (result_id)",
        "ON DELETE SET NULL (result_id, topic_id)",
        "ON DELETE SET DEFAULT (\"result_id\")",
    ] {
        let mut parsed = tokens(sql);
        rewrite_referential_set_column_lists(&mut parsed);
        assert!(
            parsed.iter().all(|token| !matches!(token, Token::LParen)),
            "{sql}: {parsed:?}"
        );
        assert!(
            parsed
                .iter()
                .any(|token| keyword_of(token) == Some(Keyword::SET)),
            "{sql}"
        );
    }
}

#[test]
fn leaves_non_referential_parentheses() {
    for sql in [
        "ON DELETE SET NULL",
        "ON DELETE CASCADE",
        "ON CONFLICT DO NOTHING",
        "ALTER COLUMN result_id SET DEFAULT (gen_random_uuid())",
        "REFERENCES parent (result_id) ON DELETE SET NULL",
        "ON UPDATE SET NULL (result_id)",
        "ON UPDATE SET DEFAULT (result_id, topic_id)",
        "ON DELETE SET NULL (result_id",
    ] {
        let mut parsed = tokens(sql);
        let before = parsed.clone();
        rewrite_referential_set_column_lists(&mut parsed);
        assert_eq!(parsed, before, "{sql}");
    }
}

#[test]
fn leaves_chr_without_call_parens_or_a_numeric_arg() {
    for sql in ["ESCAPE chr 92", "ESCAPE chr(id)"] {
        let mut parsed = tokens(sql);
        rewrite_chr_calls(&mut parsed);
        assert!(
            parsed.iter().any(|token| matches!(
                token,
                Token::Word(word) if word.value.eq_ignore_ascii_case("chr")
            )),
            "{sql}"
        );
    }
}
