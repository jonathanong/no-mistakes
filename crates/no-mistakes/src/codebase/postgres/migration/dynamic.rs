//! Statically recoverable SQL embedded in executable PL/pgSQL routine bodies.
//!
//! PostgreSQL's tokenizer, rather than a source regex, owns comments, escapes,
//! dollar tags, and semicolons in strings.
use sqlparser::tokenizer::{Token, TokenWithSpan};

mod expression;
mod literal;
mod routine;

use routine::RoutineBody;

#[derive(Clone, Debug)]
pub(super) struct DynamicSql {
    pub(super) sql: String,
    pub(super) line: usize,
    source_lines: Vec<usize>,
}

impl DynamicSql {
    fn anchored(sql: String, line: usize) -> Self {
        let line_count = sql.bytes().filter(|byte| *byte == b'\n').count() + 1;
        Self {
            sql,
            line,
            source_lines: vec![line; line_count],
        }
    }

    pub(super) fn source_line(&self, decoded_line: usize) -> usize {
        self.source_lines
            .get(decoded_line.saturating_sub(1))
            .copied()
            .or_else(|| self.source_lines.last().copied())
            .unwrap_or(self.line)
    }
}

pub(super) fn extract(sql: &str) -> Vec<DynamicSql> {
    expression::extract(sql)
}

pub(super) fn schema_bodies(sql: &str) -> Vec<DynamicSql> {
    routine::schema_bodies(sql)
}

fn tokenize(sql: &str) -> Vec<TokenWithSpan> {
    super::super::parse::unicode::tokenize_raw_unicode(sql)
}

fn statements(tokens: &[TokenWithSpan]) -> Vec<&[TokenWithSpan]> {
    let mut result = Vec::new();
    let mut start = 0;
    for (at, token) in tokens.iter().enumerate() {
        if matches!(token.token, Token::SemiColon) {
            result.push(&tokens[start..at]);
            start = at + 1;
        }
    }
    if start < tokens.len() {
        result.push(&tokens[start..]);
    }
    result
}

fn significant(tokens: &[TokenWithSpan]) -> Vec<&TokenWithSpan> {
    tokens
        .iter()
        .filter(|token| !matches!(token.token, Token::Whitespace(_)))
        .collect()
}

fn word(token: &TokenWithSpan, expected: &str) -> bool {
    identifier(token).is_some_and(|value| value.eq_ignore_ascii_case(expected))
}

fn identifier(token: &TokenWithSpan) -> Option<&str> {
    if let Token::Word(word) = &token.token {
        Some(&word.value)
    } else {
        None
    }
}

fn body_line(body: &RoutineBody, token: &TokenWithSpan) -> usize {
    location_offset(&body.sql, token.span.start.line, token.span.start.column)
        .and_then(|offset| body.source_bytes.get(offset).copied())
        .unwrap_or(body.line)
}

fn location_offset(sql: &str, line: u64, column: u64) -> Option<usize> {
    let line_start = sql
        .split_inclusive('\n')
        .take(line.saturating_sub(1) as usize)
        .map(str::len)
        .sum::<usize>();
    let current_line = sql.get(line_start..)?.split('\n').next()?;
    let column_offset = if column <= 1 {
        0
    } else {
        current_line
            .char_indices()
            .nth(column.saturating_sub(1) as usize)
            .map_or(current_line.len(), |(offset, _)| offset)
    };
    Some(line_start + column_offset)
}

fn source_lines(sql: &str, source_bytes: &[usize], fallback: usize) -> Vec<usize> {
    let mut lines = vec![source_bytes.first().copied().unwrap_or(fallback)];
    for (offset, byte) in sql.bytes().enumerate() {
        if byte == b'\n' {
            lines.push(
                source_bytes
                    .get(offset + 1)
                    .or_else(|| source_bytes.get(offset))
                    .copied()
                    .unwrap_or(fallback),
            );
        }
    }
    lines
}

fn plpgsql(tokens: &[&TokenWithSpan]) -> bool {
    tokens
        .windows(2)
        .any(|pair| word(pair[0], "LANGUAGE") && word(pair[1], "plpgsql"))
}

#[cfg(test)]
mod tests;
