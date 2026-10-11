use crate::codebase::postgres::embedded::{
    placeholders::{count_placeholders, renumber_placeholders},
    source_positions, EmbeddedSqlSourcePosition,
};
use crate::codebase::ts_source::byte_offset_to_line;
use oxc_ast::ast::Expression;
use oxc_span::GetSpan;
use std::path::Path;

/// Recovered SQL and its physical positions, carried through helper resolution.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct RecoveredSql {
    pub(crate) sql: String,
    pub(crate) line: u32,
    pub(crate) positions: Vec<EmbeddedSqlSourcePosition>,
}

impl RecoveredSql {
    pub(crate) fn with_source_file(mut self, path: &Path) -> Self {
        for position in &mut self.positions {
            position.source_path = Some(path.to_path_buf());
        }
        self
    }

    pub(crate) fn literal(expr: &Expression<'_>, source: &str, sql: String) -> Self {
        let line = byte_offset_to_line(source, expr.span().start as usize);
        let mut positions =
            source_positions::for_expression(expr, source, expr.span().start as usize, line);
        positions.insert(
            0,
            EmbeddedSqlSourcePosition {
                sql_line: 1,
                sql_column: 1,
                source_line: line,
                source_path: None,
            },
        );
        Self {
            sql,
            line,
            positions,
        }
    }

    pub(crate) fn append(&mut self, suffix: Self) {
        let offset = count_placeholders(&self.sql);
        let mut positions = suffix.positions;
        super::super::append::positions::renumber(&mut positions, &suffix.sql, offset);
        super::super::append::positions::append(
            &mut self.positions,
            &self.sql,
            self.line,
            suffix.line,
            &positions,
        );
        self.sql
            .push_str(&renumber_placeholders(&suffix.sql, offset));
    }
}
