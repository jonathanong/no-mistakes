//! Recognize procedural-only chunks inside an already identified PL/pgSQL body.
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, TokenWithSpan};
use std::sync::Arc;
mod assignment;
mod commands;
mod declaration;

/// Declaration sections last across semicolon chunks, including in nested blocks.
#[derive(Default)]
pub(crate) struct BodyState {
    declarations: bool,
}

pub(super) fn filter_body(
    statements: Vec<super::LocatedStatement>,
) -> Vec<super::LocatedStatement> {
    let mut procedural = BodyState::default();
    let mut previous: Option<Arc<[TokenWithSpan]>> = None;
    let mut control = false;
    statements
        .into_iter()
        .filter(|located| {
            if let Some(tokens) = &located.chunk_tokens {
                if previous
                    .as_ref()
                    .is_none_or(|previous| !Arc::ptr_eq(previous, tokens))
                {
                    control = procedural.control_chunk(tokens);
                    previous = Some(Arc::clone(tokens));
                }
            }
            !super::is_begin_or_end(&located.statement)
                && !(located.recovery_failure.is_some() && control)
        })
        .collect()
}

impl BodyState {
    pub(crate) fn control_chunk(&mut self, tokens: &[TokenWithSpan]) -> bool {
        let code: Vec<_> = tokens
            .iter()
            .map(|token| &token.token)
            .filter(|token| !matches!(token, Token::Whitespace(_) | Token::SemiColon))
            .collect();
        let code = unlabeled(&code);
        if code.first().is_some_and(|token| word(token, "DECLARE")) {
            self.declarations = true;
            return code.len() == 1 || declaration::matches(&code[1..]);
        }
        if code.first().is_some_and(|token| word(token, "BEGIN")) {
            self.declarations = false;
        }
        if code.first().is_some_and(|token| word(token, "END")) {
            self.declarations = false;
        }
        if self.declarations && declaration::matches(code) {
            return true;
        }
        control(code)
    }
}

fn unlabeled<'a>(code: &'a [&'a Token]) -> &'a [&'a Token] {
    if let [Token::ShiftLeft, label_token, Token::ShiftRight, rest @ ..] = code {
        if label(label_token) {
            return rest;
        }
    }
    code
}

fn control(code: &[&Token]) -> bool {
    if let [Token::ShiftLeft, label_token, Token::ShiftRight, rest @ ..] = code {
        return label(label_token) && control(rest);
    }
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
        return ending(&code[1..]);
    }
    if ["IF", "ELSIF", "WHEN", "CASE"]
        .iter()
        .any(|kind| word(first, kind))
    {
        return boundary(code, "THEN").is_some_and(|at| {
            (!word(first, "IF") && !word(first, "ELSIF") || assignment::expression(&code[1..at]))
                && (at + 1 == code.len() || control(&code[at + 1..]))
        });
    }
    if ["FOR", "WHILE", "FOREACH"]
        .iter()
        .any(|kind| word(first, kind))
    {
        return boundary(code, "LOOP").is_some_and(|at| {
            (if word(first, "FOR") {
                commands::for_header(&code[..at])
            } else if word(first, "WHILE") {
                assignment::expression(&code[1..at])
            } else {
                commands::foreach_header(&code[..at])
            }) && (at + 1 == code.len() || control(&code[at + 1..]))
        });
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
        return assignment::expression(&code[1..]);
    }
    if ["EXIT", "CONTINUE", "RETURN", "RAISE", "ASSERT"]
        .iter()
        .any(|kind| word(first, kind))
    {
        return true;
    }
    assignment::matches(code) || commands::matches(code)
}

fn ending(code: &[&Token]) -> bool {
    match code {
        [] => true,
        [kind] => ["IF", "LOOP", "CASE"].iter().any(|name| word(kind, name)) || label(kind),
        [kind, name] => word(kind, "LOOP") && label(name),
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
