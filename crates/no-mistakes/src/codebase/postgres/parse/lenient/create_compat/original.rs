//! Reject PostgreSQL-reserved names before a parser clone can erase their clause.
use super::super::pg_ident;
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn valid(source: Option<&[TokenWithSpan]>) -> bool {
    let Some(source) = source else {
        return true;
    };
    let tokens: Vec<_> = source
        .iter()
        .map(|item| item.token.clone())
        .filter(|token| !matches!(token, Token::Whitespace(_)))
        .collect();
    if !tokens.first().is_some_and(|token| word(token, "CREATE")) {
        return true;
    }
    let table_at = if tokens.get(1).is_some_and(|token| word(token, "TABLE")) {
        true
    } else {
        tokens.get(1).is_some_and(|token| {
            ["TEMP", "TEMPORARY", "UNLOGGED"]
                .iter()
                .any(|name| word(token, name))
        }) && tokens.get(2).is_some_and(|token| word(token, "TABLE"))
    };
    if !table_at {
        return true;
    }
    if !relevant(&tokens) {
        return true;
    }
    let Some(after_name) = pg_ident::table_head(&tokens) else {
        return false;
    };
    if tokens.get(after_name) == Some(&Token::LParen) {
        if !column_clauses(&tokens, after_name) || !table_tail(&tokens, after_name) {
            return false;
        }
    }
    partition_columns(&tokens, after_name)
}

fn column_clauses(tokens: &[Token], open: usize) -> bool {
    let mut depth = 0usize;
    for at in open..tokens.len() {
        match tokens[at] {
            Token::LParen => depth += 1,
            Token::RParen => {
                let Some(next) = depth.checked_sub(1) else {
                    return false;
                };
                depth = next;
                if depth == 0 {
                    return true;
                }
            }
            _ => {}
        }
        if depth != 1 {
            continue;
        }
        if word(&tokens[at], "LIKE")
            && at > 0
            && matches!(tokens[at - 1], Token::LParen | Token::Comma)
            && !tokens.get(at + 1).is_some_and(pg_ident::col_id)
        {
            return false;
        }
        if word(&tokens[at], "COMPRESSION")
            && tokens
                .get(at + 2)
                .is_some_and(|token| matches!(token, Token::Comma | Token::RParen))
            && !tokens.get(at + 1).is_some_and(pg_ident::method)
        {
            return false;
        }
    }
    true // The ordinary parser diagnoses unbalanced parentheses.
}

fn relevant(tokens: &[Token]) -> bool {
    tokens.iter().enumerate().any(|(at, token)| {
        word(token, "LIKE") && at > 0 && matches!(tokens[at - 1], Token::LParen | Token::Comma)
            || word(token, "COMPRESSION")
                && matches!(tokens.get(at + 1), Some(Token::Word(_)))
                && matches!(tokens.get(at + 2), Some(Token::Comma | Token::RParen))
            || word(token, "PARTITION") && tokens.get(at + 1).is_some_and(|token| word(token, "OF"))
            || word(token, "PARTITION") && tokens.get(at + 1).is_some_and(|token| word(token, "BY"))
            || word(token, "USING")
    })
}

fn table_tail(tokens: &[Token], open: usize) -> bool {
    let mut depth = 0usize;
    let mut close = None;
    for (at, token) in tokens.iter().enumerate().skip(open) {
        match token {
            Token::LParen => depth += 1,
            Token::RParen => {
                depth -= 1;
                if depth == 0 {
                    close = Some(at);
                    break;
                }
            }
            _ => {}
        }
    }
    let Some(close) = close else {
        return true; // The ordinary parser diagnoses the missing closing parenthesis.
    };
    let at = close + 1;
    (!tokens.get(at).is_some_and(|token| word(token, "USING"))
        || tokens.get(at + 1).is_some_and(pg_ident::method))
        && !(tokens.get(at).is_some_and(|token| word(token, "PARTITION"))
            && tokens.get(at + 1).is_some_and(|token| word(token, "BY"))
            && tokens.get(at + 3) == Some(&Token::LParen)
            && tokens.get(at + 4) == Some(&Token::RParen))
}

fn partition_columns(tokens: &[Token], after_name: usize) -> bool {
    let Some(at) = (after_name..tokens.len().saturating_sub(2))
        .find(|at| word(&tokens[*at], "PARTITION") && word(&tokens[*at + 1], "OF"))
    else {
        return true;
    };
    if !pg_ident::col_id(&tokens[at + 2]) {
        return false;
    }
    let mut at = at + 3;
    while tokens.get(at) == Some(&Token::Period) {
        if !tokens.get(at + 1).is_some_and(pg_ident::attribute) {
            return false;
        }
        at += 2;
    }
    if tokens.get(at) != Some(&Token::LParen) {
        return true;
    }
    at += 1;
    if !tokens.get(at).is_some_and(pg_ident::col_id) {
        return false;
    }
    at += 1;
    while tokens.get(at) == Some(&Token::Comma) {
        at += 1;
        if !tokens.get(at).is_some_and(pg_ident::col_id) {
            return false;
        }
        at += 1;
    }
    tokens.get(at) == Some(&Token::RParen)
}

fn word(token: &Token, expected: &str) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(expected))
}
