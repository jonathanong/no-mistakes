//! Validate PostgreSQL opclass parameters before hiding them from sqlparser.
use crate::codebase::postgres::parse::lenient::pg_ident;
use sqlparser::tokenizer::Token;

pub(super) fn options(tokens: &mut Vec<Token>) -> bool {
    let Some(using) = tokens.iter().position(|token| word(token, "USING")) else {
        return false;
    };
    let key = using + 3;
    if tokens.get(using + 2) != Some(&Token::LParen)
        || !tokens.get(using + 1).is_some_and(pg_ident::col_id)
        || !tokens.get(key).is_some_and(pg_ident::col_id)
        || !tokens.get(key + 1).is_some_and(pg_ident::col_id)
        || tokens.get(key + 2) != Some(&Token::LParen)
    {
        return false;
    }
    let mut at = key + 3;
    loop {
        if !tokens.get(at).is_some_and(pg_ident::col_id)
            || tokens.get(at + 1) != Some(&Token::Eq)
            || !tokens.get(at + 2).is_some_and(|token| {
                matches!(token, Token::Number(_, _) | Token::SingleQuotedString(_))
            })
        {
            return false;
        }
        at += 3;
        if tokens.get(at) != Some(&Token::Comma) {
            break;
        }
        at += 1;
    }
    if tokens.get(at) != Some(&Token::RParen) || tokens.get(at + 1) != Some(&Token::RParen) {
        return false;
    }
    tokens.drain(key + 2..=at);
    true
}

fn word(token: &Token, expected: &str) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_none()
        && word.value.eq_ignore_ascii_case(expected))
}
