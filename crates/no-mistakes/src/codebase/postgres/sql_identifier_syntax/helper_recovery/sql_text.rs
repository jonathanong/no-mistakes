//! Generated bind ranges are structural; authored `$1` or marker text is untouched.
use crate::codebase::postgres::{
    embedded::{helpers::program::Mark, walk::resolve::functions::provenance::RecoveredSql},
    EmbeddedSqlSourcePosition,
};
use std::path::Path;
mod binds;

#[derive(Clone, PartialEq, Eq)]
pub(super) struct SqlText {
    pub raw: RecoveredSql,
    pub binds: Vec<(usize, usize)>,
    pub executor_payload: bool,
}
impl SqlText {
    pub fn valid_binds(&self) -> bool {
        binds::valid(self)
    }
    pub fn literal(raw: RecoveredSql) -> Self {
        Self {
            raw,
            binds: Vec::new(),
            executor_payload: false,
        }
    }
    pub fn bind(mark: Mark, file: &Path, executor_payload: bool) -> Self {
        Self {
            raw: RecoveredSql {
                sql: "$1".into(),
                line: mark.line,
                origins: vec![mark.origin; 2],
                positions: vec![EmbeddedSqlSourcePosition {
                    sql_line: 1,
                    sql_column: 1,
                    source_line: mark.line,
                    source_path: Some(file.to_path_buf()),
                }],
            },
            binds: vec![(0, 2)],
            executor_payload,
        }
    }
    pub fn append(&mut self, mut suffix: Self) -> Option<()> {
        suffix.renumber(self.binds.len())?;
        self.executor_payload |= suffix.executor_payload;
        let prefix_bytes = self.raw.sql.len();
        let (line, column) = coordinate(&self.raw.sql, prefix_bytes)?;
        self.raw
            .positions
            .extend(suffix.raw.positions.into_iter().map(|mut position| {
                if position.sql_line == 1 {
                    position.sql_column += column - 1;
                }
                position.sql_line += line - 1;
                position
            }));
        self.raw.sql.push_str(&suffix.raw.sql);
        self.raw.origins.extend(suffix.raw.origins);
        self.binds.extend(
            suffix
                .binds
                .into_iter()
                .map(|(start, end)| (prefix_bytes + start, prefix_bytes + end)),
        );
        Some(())
    }
    fn renumber(&mut self, offset: usize) -> Option<()> {
        if offset == 0 || self.binds.is_empty() {
            return Some(());
        }
        let mut output = String::new();
        let mut origins = Vec::new();
        let mut ranges = Vec::new();
        let mut edits = Vec::new();
        let mut cursor = 0;
        for (index, &(start, end)) in self.binds.iter().enumerate() {
            output.push_str(self.raw.sql.get(cursor..start)?);
            origins.extend_from_slice(self.raw.origins.get(cursor..start)?);
            let replacement = format!("${}", offset + index + 1);
            let origin = *self.raw.origins.get(start)?;
            let next = output.len();
            output.push_str(&replacement);
            origins.extend(std::iter::repeat_n(origin, replacement.len()));
            ranges.push((next, output.len()));
            edits.push((start, end, next, output.len()));
            cursor = end;
        }
        output.push_str(self.raw.sql.get(cursor..)?);
        origins.extend_from_slice(self.raw.origins.get(cursor..)?);
        for position in &mut self.raw.positions {
            let old = byte_offset(&self.raw.sql, position.sql_line, position.sql_column)?;
            let mut mapped = old;
            for &(start, end, next, next_end) in &edits {
                if old < start {
                    break;
                }
                if old < end {
                    mapped = next;
                    break;
                }
                mapped = next_end.checked_add(old - end)?;
            }
            (position.sql_line, position.sql_column) = coordinate(&output, mapped)?;
        }
        self.raw.sql = output;
        self.raw.origins = origins;
        self.binds = ranges;
        Some(())
    }
}
fn coordinate(text: &str, offset: usize) -> Option<(u32, u32)> {
    let prefix = text.get(..offset)?;
    Some((
        prefix.bytes().filter(|byte| *byte == b'\n').count() as u32 + 1,
        prefix.rsplit('\n').next()?.chars().count() as u32 + 1,
    ))
}
fn byte_offset(text: &str, line: u32, column: u32) -> Option<usize> {
    let mut at = (1, 1);
    for (offset, character) in text.char_indices() {
        if at == (line, column) {
            return Some(offset);
        }
        if character == '\n' {
            at.0 += 1;
            at.1 = 1;
        } else {
            at.1 += 1;
        }
    }
    (at == (line, column)).then_some(text.len())
}
