//! Borrowed SQL tokens and source spans shared by row-cap fact consumers.
use sqlparser::tokenizer::{Location, Span, TokenWithSpan};
use std::cell::OnceCell;

/// One source borrowed from prepared analysis. Parsing, row-bound identity, and sweeps
/// inspect the same located tokens while literal text resolves against the original SQL.
pub(super) struct Tokens<'a> {
    sql: &'a str,
    prepared: &'a [TokenWithSpan],
    positions: OnceCell<Vec<SourceLine>>,
}

impl<'a> Tokens<'a> {
    pub(super) fn with_prepared(sql: &'a str, prepared: &'a [TokenWithSpan]) -> Self {
        Self {
            sql,
            prepared,
            positions: OnceCell::new(),
        }
    }

    pub(super) fn all(&self) -> &[TokenWithSpan] {
        self.prepared
    }

    /// Sparse Unicode checkpoints bound every endpoint scan to 63 characters, including
    /// many literals on one long line. Both source and index stay request-local.
    pub(super) fn source_at(&self, span: Span) -> Option<&'a str> {
        let positions = self.positions.get_or_init(|| source_positions(self.sql));
        let start = byte_offset(self.sql, positions, span.start)?;
        let end = byte_offset(self.sql, positions, span.end)?;
        self.sql.get(start..end)
    }
}

const CHECKPOINT_WIDTH: usize = 64;

#[derive(Default)]
struct SourceLine {
    checkpoints: Vec<usize>,
    columns: usize,
}

impl SourceLine {
    fn push(&mut self, offset: usize) {
        if self.columns.is_multiple_of(CHECKPOINT_WIDTH) {
            self.checkpoints.push(offset);
        }
        self.columns += 1;
    }
}

fn source_positions(sql: &str) -> Vec<SourceLine> {
    let mut lines = vec![SourceLine::default()];
    for (offset, character) in sql.char_indices() {
        lines.last_mut().unwrap().push(offset);
        if character == '\n' {
            lines.push(SourceLine::default());
        }
    }
    lines.last_mut().unwrap().push(sql.len());
    lines
}

fn byte_offset(sql: &str, positions: &[SourceLine], target: Location) -> Option<usize> {
    if target.line == 0 || target.line > positions.len() as u64 {
        return None;
    }
    let line = &positions[(target.line - 1) as usize];
    if target.column == 0 || target.column > line.columns as u64 {
        return None;
    }
    let column = (target.column - 1) as usize;
    let checkpoint = line.checkpoints[column / CHECKPOINT_WIDTH];
    let remainder = column % CHECKPOINT_WIDTH;
    // Appending the EOF endpoint lets a valid final location resolve without a character.
    let relative = sql[checkpoint..]
        .char_indices()
        .map(|(offset, _)| offset)
        .chain(std::iter::once(sql.len() - checkpoint))
        .nth(remainder)
        .unwrap();
    Some(checkpoint + relative)
}

#[cfg(test)]
mod tests;
