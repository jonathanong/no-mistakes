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
        "FOREIGN KEY (result_id) REFERENCES parent (id) ON DELETE SET NULL (result_id)",
        "FOREIGN KEY (result_id, topic_id) REFERENCES parent (result_id, topic_id) ON DELETE SET NULL (result_id, topic_id)",
        "FOREIGN KEY (result_id) REFERENCES parent (id) ON DELETE SET DEFAULT (\"result_id\")",
        "FOREIGN KEY (\"NULL\") REFERENCES parent (id) ON DELETE SET NULL (\"NULL\")",
        "FOREIGN KEY (key) REFERENCES parent (id) ON DELETE SET NULL (key)",
        "FOREIGN KEY (topic_id, result_id) REFERENCES parent (topic_id, result_id) ON DELETE SET NULL (result_id)",
    ] {
        let mut parsed = tokens(sql);
        rewrite_referential_set_column_lists(&mut parsed);
        assert!(action_column_list_removed(&parsed), "{sql}: {parsed:?}");
    }
}

#[test]
fn leaves_non_referential_parentheses() {
    for sql in [
        "ON DELETE SET NULL",
        "ON DELETE CASCADE",
        "ON DELETE SET RESTRICT",
        "ON CONFLICT DO NOTHING",
        "ALTER COLUMN result_id SET DEFAULT (gen_random_uuid())",
        "REFERENCES parent (result_id) ON DELETE SET NULL",
        "ON UPDATE SET NULL (result_id)",
        "ON UPDATE SET DEFAULT (result_id, topic_id)",
        "ON DELETE SET NULL ()",
        "ON DELETE SET NULL (1)",
        "ON DELETE SET NULL (NULL)",
        "ON DELETE SET NULL (CURRENT_DATE)",
        "ON DELETE SET NULL (ARRAY)",
        "ON DELETE SET NULL (ANALYZE)",
        "ON DELETE SET NULL (ISNULL)",
        "FOREIGN KEY (topic_id, result_id) REFERENCES parent (topic_id, result_id) ON DELETE SET NULL (other_id)",
        "ON DELETE SET NULL (key)",
        "ON DELETE SET NULL (result_id + 1)",
        "ON DELETE SET NULL (result_id,)",
        "ON DELETE SET NULL (result_id",
    ] {
        let mut parsed = tokens(sql);
        let before = parsed.clone();
        rewrite_referential_set_column_lists(&mut parsed);
        assert_eq!(parsed, before, "{sql}");
    }
}

fn action_column_list_removed(tokens: &[Token]) -> bool {
    let Some(value_at) = tokens.iter().rposition(|token| {
        matches!(keyword_of(token), Some(Keyword::NULL | Keyword::DEFAULT))
            && token_is_unquoted(token)
    }) else {
        return false;
    };
    tokens
        .iter()
        .skip(value_at + 1)
        .find(|token| !matches!(token, Token::Whitespace(_)))
        .is_none_or(|token| !matches!(token, Token::LParen))
}

fn token_is_unquoted(token: &Token) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_none())
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
