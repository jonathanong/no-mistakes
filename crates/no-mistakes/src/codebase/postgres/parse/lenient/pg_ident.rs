//! PostgreSQL 17 identifier classes at compatibility rewrite boundaries.
use super::recover::failure::keywords;
use sqlparser::tokenizer::Token;

pub(in crate::codebase::postgres::parse) fn col_id(token: &Token) -> bool {
    matches!(token, Token::Word(word) if !word.value.is_empty()
        && (word.quote_style == Some('"')
            || (word.quote_style.is_none() && !keywords::reserved(&word.value))))
}

pub(super) fn method(token: &Token) -> bool {
    col_id(token) || word(token, "DEFAULT")
}

/// Index following CREATE [TEMP|UNLOGGED] TABLE [IF NOT EXISTS] qualified_name.
pub(super) fn table_head(tokens: &[Token]) -> Option<usize> {
    let mut at = 0;
    if !word(tokens.get(at)?, "CREATE") {
        return None;
    }
    at += 1;
    if tokens.get(at).is_some_and(|token| {
        ["TEMP", "TEMPORARY", "UNLOGGED"]
            .iter()
            .any(|name| word(token, name))
    }) {
        at += 1;
    }
    if !word(tokens.get(at)?, "TABLE") {
        return None;
    }
    at += 1;
    if tokens.get(at).is_some_and(|token| word(token, "IF")) {
        if !word(tokens.get(at + 1)?, "NOT") || !word(tokens.get(at + 2)?, "EXISTS") {
            return None;
        }
        at += 3;
    }
    if !col_id(tokens.get(at)?) {
        return None;
    }
    at += 1;
    while tokens.get(at) == Some(&Token::Period) {
        if !attribute(tokens.get(at + 1)?) {
            return None;
        }
        at += 2;
    }
    Some(at)
}

pub(super) fn attribute(token: &Token) -> bool {
    matches!(token, Token::Word(word) if !word.value.is_empty()
        && (word.quote_style.is_none() || word.quote_style == Some('"')))
}

fn word(token: &Token, expected: &str) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(expected))
}
