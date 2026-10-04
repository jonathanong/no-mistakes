use super::{PostgresSqlPosition, PostgresSqlSpan};
use sqlparser::tokenizer::{Location, Span};

/// Unicode character boundaries indexed once for this source request.
pub(super) struct Locations<'a> {
    sql: &'a str,
    lines: Vec<Vec<usize>>,
}

impl<'a> Locations<'a> {
    pub(super) fn new(sql: &'a str) -> Self {
        let mut lines = vec![Vec::new()];
        for (offset, character) in sql.char_indices() {
            lines.last_mut().unwrap().push(offset);
            if character == '\n' {
                lines.push(Vec::new());
            }
        }
        lines.last_mut().unwrap().push(sql.len());
        Self { sql, lines }
    }

    pub(super) fn position(&self, location: Location) -> Option<PostgresSqlPosition> {
        let line = usize::try_from(location.line).ok()?.checked_sub(1)?;
        let column = usize::try_from(location.column).ok()?.checked_sub(1)?;
        let offset = *self.lines.get(line)?.get(column)?;
        Some(PostgresSqlPosition {
            offset,
            line: line + 1,
            column: column + 1,
        })
    }

    pub(super) fn span(&self, span: Span) -> Option<PostgresSqlSpan> {
        let start = self.position(span.start)?;
        let end = self.position(span.end)?;
        (start.offset <= end.offset).then_some(PostgresSqlSpan { start, end })
    }

    pub(super) fn range(&self, start: usize, end: usize) -> PostgresSqlSpan {
        let start = self.boundary(start);
        let end = self.boundary(end).max(start);
        PostgresSqlSpan {
            start: self.at_offset(start),
            end: self.at_offset(end),
        }
    }

    fn boundary(&self, offset: usize) -> usize {
        let mut offset = offset.min(self.sql.len());
        while !self.sql.is_char_boundary(offset) {
            offset -= 1;
        }
        offset
    }

    fn at_offset(&self, offset: usize) -> PostgresSqlPosition {
        let line = self.lines.partition_point(|line| line[0] <= offset) - 1;
        let column = self.lines[line].binary_search(&offset).unwrap();
        PostgresSqlPosition {
            offset,
            line: line + 1,
            column: column + 1,
        }
    }
}
