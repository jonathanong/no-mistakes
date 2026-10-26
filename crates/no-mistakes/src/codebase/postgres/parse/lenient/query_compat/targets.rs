//! Locate syntactic DML targets before removing unsupported target selectors.
use super::word;
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn insert_targets(tokens: &[TokenWithSpan], code: &[usize]) -> Vec<usize> {
    if code.len() < 5 || !word(&tokens[code[1]].token, "INTO") {
        return Vec::new();
    }
    let mut at = 2;
    if !identifier(tokens, code, at) {
        return Vec::new();
    }
    at += 1;
    while token(tokens, code, at) == Some(&Token::Period) {
        if !identifier(tokens, code, at + 1) {
            return Vec::new();
        }
        at += 2;
    }
    if code
        .get(at)
        .is_some_and(|index| word(&tokens[*index].token, "AS"))
    {
        at += 1;
        if !identifier(tokens, code, at) {
            return Vec::new();
        }
        at += 1;
    } else if identifier(tokens, code, at) && token(tokens, code, at + 1) == Some(&Token::LParen) {
        at += 1;
    }
    if token(tokens, code, at) != Some(&Token::LParen) {
        return Vec::new();
    }
    let mut names = Vec::new();
    let mut parens = 1usize;
    let mut brackets = 0usize;
    at += 1;
    if identifier(tokens, code, at) {
        names.push(at);
    }
    while at < code.len() {
        match token(tokens, code, at) {
            Some(Token::LParen) => parens += 1,
            Some(Token::RParen) => {
                parens -= 1;
                if parens == 0 {
                    return names;
                }
            }
            Some(Token::LBracket) => brackets += 1,
            Some(Token::RBracket) if brackets > 0 => brackets -= 1,
            Some(Token::Comma)
                if parens == 1 && brackets == 0 && identifier(tokens, code, at + 1) =>
            {
                names.push(at + 1);
            }
            _ => {}
        }
        at += 1;
    }
    Vec::new()
}

pub(super) fn assignment_targets(tokens: &[TokenWithSpan], code: &[usize]) -> Vec<usize> {
    let Some(start) = top_level_set(tokens, code) else {
        return Vec::new();
    };
    let mut names = Vec::new();
    if identifier(tokens, code, start + 1) {
        names.push(start + 1);
    }
    let mut parens = 0usize;
    let mut brackets = 0usize;
    for at in start + 1..code.len() {
        let next = token(tokens, code, at);
        if parens == 0 && brackets == 0 {
            if ["WHERE", "FROM", "RETURNING"].iter().any(|end| {
                code.get(at)
                    .is_some_and(|index| word(&tokens[*index].token, end))
            }) {
                break;
            }
            if next == Some(&Token::Comma) && identifier(tokens, code, at + 1) {
                names.push(at + 1);
            }
        }
        match next {
            Some(Token::LParen) => parens += 1,
            Some(Token::RParen) if parens > 0 => parens -= 1,
            Some(Token::LBracket) => brackets += 1,
            Some(Token::RBracket) if brackets > 0 => brackets -= 1,
            _ => {}
        }
    }
    names
}

fn top_level_set(tokens: &[TokenWithSpan], code: &[usize]) -> Option<usize> {
    let mut depth = 0usize;
    let update = code
        .first()
        .is_some_and(|first| word(&tokens[*first].token, "UPDATE"));
    for (at, index) in code.iter().copied().enumerate() {
        let next = &tokens[index].token;
        let conflict_update = at >= 2
            && word(&tokens[code[at - 2]].token, "DO")
            && word(&tokens[code[at - 1]].token, "UPDATE");
        if depth == 0 && word(next, "SET") && (update || conflict_update) {
            return Some(at);
        }
        match next {
            Token::LParen | Token::LBracket => depth += 1,
            Token::RParen | Token::RBracket if depth > 0 => depth -= 1,
            _ => {}
        }
    }
    None
}
fn identifier(tokens: &[TokenWithSpan], code: &[usize], at: usize) -> bool {
    matches!(token(tokens, code, at), Some(Token::Word(_)))
}
fn token<'a>(tokens: &'a [TokenWithSpan], code: &[usize], at: usize) -> Option<&'a Token> {
    code.get(at).map(|index| &tokens[*index].token)
}
