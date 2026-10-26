use super::super::pg_ident;
use sqlparser::tokenizer::{Token, TokenWithSpan, Whitespace};

fn mask(tokens: &mut [TokenWithSpan], code: &[usize], start: usize, end: usize) {
    for at in &code[start..end] {
        tokens[*at].token = Token::Whitespace(Whitespace::Space);
    }
}

#[path = "clauses/modifiers.rs"]
mod modifiers;
pub(super) use modifiers::column_modifiers;

fn at_column_depth(tokens: &[TokenWithSpan], code: &[usize], body: usize, at: usize) -> bool {
    if at <= body {
        return false;
    }
    let mut depth = 0usize;
    for pos in body..at {
        match tokens[code[pos]].token {
            Token::LParen => depth += 1,
            Token::RParen => {
                let Some(next) = depth.checked_sub(1) else {
                    return false;
                };
                depth = next;
            }
            _ => {}
        }
    }
    depth == 1
}

pub(super) fn partition_columns(tokens: &mut [TokenWithSpan], code: &[usize]) -> bool {
    for at in 0..code.len().saturating_sub(6) {
        if !word(&tokens[code[at]].token, "PARTITION")
            || !word(&tokens[code[at + 1]].token, "OF")
            || !pg_ident::col_id(&tokens[code[at + 2]].token)
            || tokens[code[at + 3]].token != Token::LParen
        {
            continue;
        }
        let mut end = at + 4;
        if !identifiers(tokens, code, &mut end)
            || code
                .get(end)
                .is_none_or(|pos| tokens[*pos].token != Token::RParen)
            || !code
                .get(end + 1)
                .is_some_and(|pos| word(&tokens[*pos].token, "FOR"))
        {
            continue;
        }
        mask(tokens, code, at + 3, end + 1);
        return true;
    }
    false
}

fn identifiers(tokens: &[TokenWithSpan], code: &[usize], at: &mut usize) -> bool {
    if !code
        .get(*at)
        .is_some_and(|pos| pg_ident::col_id(&tokens[*pos].token))
    {
        return false;
    }
    *at += 1;
    while code
        .get(*at)
        .is_some_and(|pos| tokens[*pos].token == Token::Comma)
    {
        *at += 1;
        if !code
            .get(*at)
            .is_some_and(|pos| pg_ident::col_id(&tokens[*pos].token))
        {
            return false;
        }
        *at += 1;
    }
    true
}

pub(super) fn generated_default(tokens: &mut [TokenWithSpan], code: &[usize]) -> bool {
    for at in 0..code.len().saturating_sub(5) {
        if ["GENERATED", "BY", "DEFAULT", "AS"]
            .iter()
            .enumerate()
            .all(|(offset, name)| word(&tokens[code[at + offset]].token, name))
            && tokens[code[at + 4]].token == Token::LParen
            && stored_after_expression(tokens, code, at + 4)
        {
            tokens[code[at + 1]].token = Token::Whitespace(Whitespace::Space);
            tokens[code[at + 2]].token = Token::make_keyword("ALWAYS");
            return true;
        }
    }
    false
}

fn stored_after_expression(tokens: &[TokenWithSpan], code: &[usize], open: usize) -> bool {
    let mut depth = 0;
    for at in open..code.len() {
        match tokens[code[at]].token {
            Token::LParen => depth += 1,
            Token::RParen => {
                depth -= 1;
                if depth == 0 {
                    return code
                        .get(at + 1)
                        .is_some_and(|next| word(&tokens[*next].token, "STORED"));
                }
            }
            _ => {}
        }
    }
    false
}

pub(super) fn table_options(tokens: &mut [TokenWithSpan], code: &[usize]) -> bool {
    for at in 1..code.len().saturating_sub(1) {
        if word(&tokens[code[at]].token, "WITHOUT")
            && word(&tokens[code[at + 1]].token, "OIDS")
            && tokens[code[at - 1]].token == Token::RParen
            && at + 2 == code.len()
        {
            mask(tokens, code, at, at + 2);
            return true;
        }
    }
    false
}

pub(super) fn word(token: &Token, expected: &str) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(expected))
}
