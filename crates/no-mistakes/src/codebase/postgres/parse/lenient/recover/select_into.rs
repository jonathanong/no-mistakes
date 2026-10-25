//! Adapt PL/pgSQL variable targets only in a parser clone of an identified body.
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn normalize(tokens: &mut Vec<TokenWithSpan>) -> bool {
    let code: Vec<_> = tokens
        .iter()
        .enumerate()
        .filter(|(_, token)| !matches!(token.token, Token::Whitespace(_)))
        .collect();
    if !code
        .first()
        .is_some_and(|(_, token)| word(&token.token, "SELECT") || word(&token.token, "WITH"))
    {
        return true;
    }
    let mut depth = 0usize;
    let mut dml = false;
    let Some(at) = code.iter().position(|(_, token)| {
        match token.token {
            Token::LParen | Token::LBracket => depth += 1,
            Token::RParen | Token::RBracket => depth = depth.saturating_sub(1),
            _ => {}
        }
        dml |= depth == 0
            && ["INSERT", "UPDATE", "DELETE", "MERGE"]
                .iter()
                .any(|name| word(&token.token, name));
        depth == 0 && word(&token.token, "INTO")
    }) else {
        return true;
    };
    // WITH may carry DML (including INSERT ... INTO), not a SELECT variable target.
    if dml {
        return true;
    }
    let mut start = at + 1;
    if code
        .get(start)
        .is_some_and(|(_, token)| word(&token.token, "STRICT"))
    {
        start += 1;
    }
    let end = code
        .iter()
        .enumerate()
        .skip(start)
        .find_map(|(index, (_, token))| {
            [
                "FROM",
                "WHERE",
                "GROUP",
                "HAVING",
                "WINDOW",
                "ORDER",
                "LIMIT",
                "OFFSET",
                "FETCH",
                "FOR",
                "UNION",
                "INTERSECT",
                "EXCEPT",
            ]
            .iter()
            .any(|name| word(&token.token, name))
            .then_some(index)
        })
        .unwrap_or(code.len());
    let targets = &code[start..end];
    let parts: Vec<_> = targets
        .split(|(_, token)| token.token == Token::Comma)
        .collect();
    if parts.iter().any(|part| !target(part)) {
        return false;
    }
    // Native SelectInto retains every target but has no STRICT field. Only
    // remove STRICT from this clone; source tokens and target spans stay intact.
    if start > at + 1 {
        tokens.remove(code[at + 1].0);
    }
    true
}

fn target(code: &[(usize, &TokenWithSpan)]) -> bool {
    let tokens: Vec<_> = code.iter().map(|(_, token)| &token.token).collect();
    super::procedural::assignment_target(&tokens)
}

fn word(token: &Token, name: &str) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests;
