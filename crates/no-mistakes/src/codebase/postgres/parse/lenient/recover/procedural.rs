//! Recognize procedural-only chunks inside an already identified PL/pgSQL body.
use sqlparser::keywords::Keyword;
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
                && (["IF", "LOOP", "CASE"]
                    .iter()
                    .any(|kind| word(code[1], kind))
                    || label(code[1]))
            || code.len() == 3 && word(code[1], "LOOP") && label(code[2]);
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
    if word(first, "PERFORM") {
        return simple_perform(&code[1..]);
    }
    if ["EXIT", "CONTINUE", "RETURN", "RAISE", "ASSERT"]
        .iter()
        .any(|kind| word(first, kind))
    {
        return true;
    }
    if word(first, "DECLARE") {
        return !code
            .iter()
            .any(|token| word(token, "CURSOR") || word(token, "FOR"));
    }
    if word(first, "OPEN") {
        return code.len() == 2 && label(code[1]);
    }
    if word(first, "GET") {
        return code.get(1).is_some_and(|token| word(token, "DIAGNOSTICS"))
            || code.get(1).is_some_and(|token| word(token, "STACKED"))
                && code.get(2).is_some_and(|token| word(token, "DIAGNOSTICS"));
    }
    if word(first, "CLOSE") {
        return code.len() == 2 && label(code[1]);
    }
    false
}

fn simple_perform(code: &[&Token]) -> bool {
    match code {
        [Token::Number(_, _)] | [Token::SingleQuotedString(_)] | [Token::Word(_)] => true,
        [Token::Word(_), Token::LParen, ..] => {
            let mut depth = 0usize;
            for (at, token) in code.iter().enumerate().skip(1) {
                match token {
                    Token::LParen => depth += 1,
                    Token::RParen => {
                        depth = depth.saturating_sub(1);
                        if depth == 0 {
                            return at + 1 == code.len();
                        }
                    }
                    _ => {}
                }
            }
            false
        }
        _ => false,
    }
}

fn label(token: &Token) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_some() || word.keyword == Keyword::NoKeyword)
}

fn boundary(code: &[&Token], expected: &str) -> Option<usize> {
    let mut depth = 0usize;
    let mut cases = 0usize;
    for (at, token) in code.iter().enumerate().skip(1) {
        match token {
            Token::LParen => depth += 1,
            Token::RParen => depth = depth.saturating_sub(1),
            _ if word(token, "CASE") => cases += 1,
            _ if word(token, "END") => cases = cases.saturating_sub(1),
            _ if depth == 0 && cases == 0 && word(token, expected) => return Some(at),
            _ => {}
        }
    }
    None
}

fn word(token: &Token, expected: &str) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(expected))
}
