//! Validate SQL-bearing PL/pgSQL commands only after a routine body is identified.
use super::{assignment, commands, word};
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, Word};

pub(super) fn perform(code: &[&Token]) -> bool {
    // PERFORM discards results; SELECT's INTO clause is not legal here.
    let mut depth = 0usize;
    for token in code {
        match token {
            Token::LParen | Token::LBracket => depth += 1,
            Token::RParen | Token::RBracket => depth = depth.saturating_sub(1),
            _ if depth == 0 && word(token, "INTO") => return false,
            _ => {}
        }
    }
    let select = Token::Word(Word {
        value: "SELECT".into(),
        quote_style: None,
        keyword: Keyword::SELECT,
    });
    let mut query = vec![&select];
    query.extend_from_slice(code);
    commands::query(&query)
}

pub(super) fn matches(code: &[&Token]) -> bool {
    if word(code[0], "SELECT") || word(code[0], "WITH") {
        return commands::query(code);
    }
    // Literal EXECUTE payloads retain their existing static extraction/failure path.
    // Unknown runtime SQL must remain opaque; only the explicit format form is
    // a recognized procedural command here.
    if !word(code[0], "EXECUTE")
        || !code.get(1).is_some_and(|token| word(token, "format"))
        || !matches!(code.get(2), Some(Token::LParen))
        || !matches!(
            code.get(3),
            Some(Token::SingleQuotedString(_) | Token::DollarQuotedString(_))
        )
    {
        return false;
    }
    // INTO and USING delimit the dynamic SQL expression only at its outer level.
    let mut depth = 0usize;
    let mut end = code.len();
    for (at, token) in code.iter().enumerate().skip(1) {
        match token {
            Token::LParen | Token::LBracket => depth += 1,
            Token::RParen | Token::RBracket => depth = depth.saturating_sub(1),
            _ if depth == 0 && (word(token, "INTO") || word(token, "USING")) => {
                end = at;
                break;
            }
            _ => {}
        }
    }
    if !assignment::expression(&code[1..end]) {
        return false;
    }
    let mut tail = &code[end..];
    if tail.first().is_some_and(|token| word(token, "INTO")) {
        tail = &tail[1..];
        if tail.first().is_some_and(|token| word(token, "STRICT")) {
            tail = &tail[1..];
        }
        let using = tail
            .iter()
            .position(|token| word(token, "USING"))
            .unwrap_or(tail.len());
        if !assignment::comma_parts(&tail[..using])
            .is_some_and(|parts| parts.into_iter().all(assignment::target))
        {
            return false;
        }
        tail = &tail[using..];
    }
    tail.is_empty()
        || word(tail[0], "USING")
            && assignment::comma_parts(&tail[1..])
                .is_some_and(|parts| parts.into_iter().all(assignment::expression))
}
