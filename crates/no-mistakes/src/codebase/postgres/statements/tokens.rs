//! Borrowed SQL tokens and source spans shared by row-cap fact consumers.
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::tokenizer::{Location, Span, TokenWithSpan, Tokenizer};
use std::cell::OnceCell;

/// One source borrowed from prepared analysis: tokens are produced once on first use, and
/// numeric literal identity is recovered directly from the same source.
pub(super) struct Tokens<'a> {
    sql: &'a str,
    tokens: OnceCell<Vec<TokenWithSpan>>,
    positions: OnceCell<Vec<Vec<usize>>>,
}

impl<'a> Tokens<'a> {
    pub(super) fn new(sql: &'a str) -> Self {
        Self {
            sql,
            tokens: OnceCell::new(),
            positions: OnceCell::new(),
        }
    }

    pub(super) fn all(&self) -> &[TokenWithSpan] {
        self.tokens.get_or_init(|| {
            Tokenizer::new(&PostgreSqlDialect {}, self.sql)
                .tokenize_with_location()
                .unwrap_or_default()
        })
    }

    /// Index Unicode scalar positions once, so every endpoint lookup is constant-time even
    /// when many literals share one long line. Both source and index stay request-local.
    pub(super) fn source_at(&self, span: Span) -> Option<&'a str> {
        let positions = self.positions.get_or_init(|| source_positions(self.sql));
        let start = byte_offset(positions, span.start)?;
        let end = byte_offset(positions, span.end)?;
        self.sql.get(start..end)
    }
}

fn source_positions(sql: &str) -> Vec<Vec<usize>> {
    let mut lines = vec![Vec::new()];
    for (offset, character) in sql.char_indices() {
        lines.last_mut().unwrap().push(offset);
        if character == '\n' {
            lines.push(Vec::new());
        }
    }
    lines.last_mut().unwrap().push(sql.len());
    lines
}

fn byte_offset(positions: &[Vec<usize>], target: Location) -> Option<usize> {
    if target.line == 0 || target.line > positions.len() as u64 {
        return None;
    }
    let columns = &positions[(target.line - 1) as usize];
    if target.column == 0 || target.column > columns.len() as u64 {
        return None;
    }
    Some(columns[(target.column - 1) as usize])
}

#[cfg(test)]
mod tests;
