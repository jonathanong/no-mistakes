use crate::codebase::postgres::parse::lenient::pg_ident;
use sqlparser::tokenizer::Token;

pub(super) fn rewrite(tokens: &mut Vec<Token>) -> bool {
    like_options(tokens)
        || unique_nulls(tokens)
        || check_not_valid(tokens)
        || fk_set_columns(tokens)
}

fn like_options(tokens: &mut Vec<Token>) -> bool {
    for at in 0..tokens.len().saturating_sub(4) {
        if !word(&tokens[at], "LIKE") || !pg_ident::col_id(&tokens[at + 1]) {
            continue;
        }
        let mut end = at + 2;
        while end + 1 < tokens.len()
            && word(&tokens[end], "INCLUDING")
            && [
                "DEFAULTS",
                "CONSTRAINTS",
                "IDENTITY",
                "GENERATED",
                "INDEXES",
                "STORAGE",
                "COMMENTS",
                "STATISTICS",
                "COMPRESSION",
                "ALL",
            ]
            .iter()
            .any(|name| word(&tokens[end + 1], name))
        {
            end += 2;
        }
        if end > at + 2 && tokens.get(end) == Some(&Token::RParen) {
            tokens.drain(at + 2..end);
            return true;
        }
    }
    false
}

fn unique_nulls(tokens: &mut Vec<Token>) -> bool {
    for at in 0..tokens.len().saturating_sub(4) {
        if ["UNIQUE", "NULLS", "NOT", "DISTINCT"]
            .iter()
            .enumerate()
            .all(|(offset, name)| word(&tokens[at + offset], name))
            && matches!(tokens[at + 4], Token::Comma | Token::RParen)
        {
            tokens.drain(at + 1..at + 4);
            return true;
        }
    }
    false
}

fn check_not_valid(tokens: &mut Vec<Token>) -> bool {
    for at in 1..tokens.len().saturating_sub(4) {
        if tokens[at - 1] == Token::RParen
            && ["NO", "INHERIT", "NOT", "VALID"]
                .iter()
                .enumerate()
                .all(|(offset, name)| word(&tokens[at + offset], name))
            && matches!(tokens[at + 4], Token::Comma | Token::RParen)
        {
            tokens.drain(at + 2..at + 4);
            return true;
        }
    }
    false
}

fn fk_set_columns(tokens: &mut Vec<Token>) -> bool {
    for at in 3..tokens.len().saturating_sub(5) {
        if !word(&tokens[at], "ON")
            || !word(&tokens[at + 1], "DELETE")
            || !word(&tokens[at + 2], "SET")
            || !(word(&tokens[at + 3], "NULL") || word(&tokens[at + 3], "DEFAULT"))
            || tokens[at + 4] != Token::LParen
            || !tokens[..at].iter().any(|t| word(t, "FOREIGN"))
            || !tokens[..at].iter().any(|t| word(t, "REFERENCES"))
        {
            continue;
        }
        let mut end = at + 5;
        if !tokens.get(end).is_some_and(pg_ident::col_id) {
            continue;
        }
        end += 1;
        while tokens.get(end) == Some(&Token::Comma) {
            end += 1;
            if !tokens.get(end).is_some_and(pg_ident::col_id) {
                return false;
            }
            end += 1;
        }
        if tokens.get(end) != Some(&Token::RParen) {
            continue;
        }
        tokens.drain(at + 4..=end);
        return true;
    }
    false
}

fn word(token: &Token, expected: &str) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(expected))
}
