//! Recover SQL expression statements through narrow PL/pgSQL prefixes.
use sqlparser::ast::Statement;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::keywords::Keyword;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::{Token, TokenWithSpan, Word};
mod loop_header;
pub(super) use loop_header::loop_action_start;

pub(super) fn recover(
    tokens: &[Token],
    original: Option<&[TokenWithSpan]>,
    procedural_body: bool,
) -> Vec<(Statement, bool)> {
    let mut at = next(tokens, 0);
    let begin = tokens
        .get(at)
        .is_some_and(|token| word(token, "BEGIN") || word(token, "ELSE"));
    if begin {
        at = next(tokens, at + 1);
    }
    let mut result = Vec::new();
    let guarded = tokens
        .get(at)
        .is_some_and(|token| word(token, "IF") || word(token, "ELSIF"));
    if guarded {
        let Some(then) = control_end(tokens, at + 1, "THEN") else {
            return result;
        };
        if let Some(condition) = select(
            &tokens[at + 1..then],
            original.map(|tokens| &tokens[at + 1..then]),
            true,
            procedural_body,
        ) {
            result.push((condition, true));
        }
        at = next(tokens, then + 1);
    }
    let mut looped = false;
    if procedural_body {
        if let Ok(Some(action)) = loop_header::loop_action_start(tokens) {
            at = action;
            looped = true;
        }
    }
    let Some(first) = tokens.get(at) else {
        return result;
    };
    let mut replace = word(first, "PERFORM") || word(first, "RETURN");
    let query = word(first, "SELECT") || word(first, "WITH");
    if !(replace || query && (begin || guarded || looped)) {
        return result;
    }
    let mut start = if replace { next(tokens, at + 1) } else { at };
    if word(first, "RETURN") && tokens.get(start).is_some_and(|token| word(token, "QUERY")) {
        start = next(tokens, start + 1);
        replace = false;
    }
    if let Some(statement) = select(
        &tokens[start..],
        original.map(|tokens| &tokens[start..]),
        replace,
        procedural_body,
    ) {
        result.push((statement, replace));
    }
    result
}

/// The action following a procedural IF/ELSIF guard, if it shares this semicolon chunk.
pub(super) fn guarded_action_start(tokens: &[Token]) -> Option<usize> {
    let mut at = next(tokens, 0);
    if tokens
        .get(at)
        .is_some_and(|token| word(token, "BEGIN") || word(token, "ELSE"))
    {
        at = next(tokens, at + 1);
    }
    if !tokens
        .get(at)
        .is_some_and(|token| word(token, "IF") || word(token, "ELSIF"))
    {
        return None;
    }
    let then = control_end(tokens, at + 1, "THEN")?;
    let action = next(tokens, then + 1);
    (action < tokens.len()).then_some(action)
}

fn select(
    tokens: &[Token],
    original: Option<&[TokenWithSpan]>,
    prepend: bool,
    procedural_body: bool,
) -> Option<Statement> {
    let dialect = PostgreSqlDialect {};
    let mut located = original.map_or_else(
        || tokens.iter().cloned().map(TokenWithSpan::wrap).collect(),
        |tokens| tokens.to_vec(),
    );
    if prepend {
        let token = Token::Word(Word {
            value: "SELECT".into(),
            quote_style: None,
            keyword: Keyword::SELECT,
        });
        let span = located.first()?.span;
        located.insert(0, TokenWithSpan::new(token, span));
    }
    super::query_locks::rewrite_located(&mut located);
    if procedural_body && !super::select_into::normalize(&mut located) {
        return None;
    }
    let mut parser = Parser::new(&dialect).with_tokens_with_locations(located);
    let statement = parser.parse_statement().ok()?;
    matches!(parser.peek_token().token, Token::EOF).then_some(statement)
}

fn next(tokens: &[Token], at: usize) -> usize {
    tokens
        .iter()
        .enumerate()
        .skip(at)
        .find(|(_, token)| !matches!(token, Token::Whitespace(_)))
        .map_or(tokens.len(), |(at, _)| at)
}

fn control_end(tokens: &[Token], start: usize, expected: &str) -> Option<usize> {
    let mut depth = 0usize;
    let mut cases = 0usize;
    for (at, token) in tokens.iter().enumerate().skip(start) {
        match token {
            Token::LParen => depth += 1,
            Token::RParen => depth = depth.saturating_sub(1),
            _ => {}
        }
        if word(token, "CASE") {
            cases += 1;
        }
        if word(token, "END") {
            cases = cases.saturating_sub(1);
        }
        if depth == 0 && cases == 0 && word(token, expected) {
            return Some(at);
        }
    }
    None
}

fn word(token: &Token, name: &str) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "queries/action_tests.rs"]
mod action_tests;

#[cfg(test)]
#[path = "queries/loop_matrix_tests.rs"]
mod loop_matrix_tests;
