//! Recognize procedural-only chunks inside an already identified PL/pgSQL body.
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(crate) fn control_chunk(tokens: &[TokenWithSpan]) -> bool {
    let code: Vec<_> = tokens
        .iter()
        .map(|token| &token.token)
        .filter(|token| !matches!(token, Token::Whitespace(_) | Token::SemiColon))
        .collect();
    control(&code)
}

fn control(code: &[&Token]) -> bool {
    let Some(first) = code.first() else {
        return false;
    };
    if word(first, "BEGIN") {
        return code.len() == 1 || control(&code[1..]);
    }
    if word(first, "ELSE") {
        return code.len() == 1 || control(&code[1..]);
    }
    if word(first, "END") {
        return code.len() == 1
            || code.len() == 2
                && ["IF", "LOOP", "CASE"]
                    .iter()
                    .any(|kind| word(code[1], kind));
    }
    if ["IF", "ELSIF", "WHEN", "CASE"]
        .iter()
        .any(|kind| word(first, kind))
    {
        return boundary(code, "THEN")
            .is_some_and(|at| at + 1 == code.len() || control(&code[at + 1..]));
    }
    if ["FOR", "WHILE", "FOREACH"]
        .iter()
        .any(|kind| word(first, kind))
    {
        return boundary(code, "LOOP")
            .is_some_and(|at| at + 1 == code.len() || control(&code[at + 1..]));
    }
    if word(first, "LOOP") || word(first, "EXCEPTION") {
        return code.len() == 1 || control(&code[1..]);
    }
    if word(first, "NULL") {
        return code.len() == 1;
    }
    // RETURN QUERY carries SQL. If that SQL is unknown, keep its marker.
    if word(first, "RETURN") && code.get(1).is_some_and(|next| word(next, "QUERY")) {
        return false;
    }
    if ["EXIT", "CONTINUE", "RETURN", "RAISE", "PERFORM", "ASSERT"]
        .iter()
        .any(|kind| word(first, kind))
    {
        return true;
    }
    if ["DECLARE", "GET", "OPEN", "FETCH", "MOVE", "CLOSE"]
        .iter()
        .any(|kind| word(first, kind))
    {
        return true;
    }
    matches!(code.get(1), Some(Token::Assignment))
}

fn boundary(code: &[&Token], expected: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (at, token) in code.iter().enumerate().skip(1) {
        match token {
            Token::LParen => depth += 1,
            Token::RParen => depth = depth.saturating_sub(1),
            _ if depth == 0 && word(token, expected) => return Some(at),
            _ => {}
        }
    }
    None
}

fn word(token: &Token, expected: &str) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(expected))
}
