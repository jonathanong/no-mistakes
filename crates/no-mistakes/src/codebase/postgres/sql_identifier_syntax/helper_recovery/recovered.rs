//! A proof result is separate from ordinary trusted-tag syntax.
use crate::codebase::postgres::embedded::walk::resolve::functions::provenance::RecoveredSql;
use std::{
    fmt,
    path::{Path, PathBuf},
};

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct ExecutionSite {
    pub file: PathBuf,
    pub start: u32,
}

#[derive(Clone, Default)]
pub(crate) struct Recovered {
    pub alternatives: Vec<RecoveredSql>,
    /// Aligned provenance for SQL executed inside a modeled helper.
    pub execution_sites: Vec<Option<ExecutionSite>>,
    /// Any unaccounted effect, value, branch or exhausted limit stays explicit.
    pub residual: bool,
    pub nonexecuting: bool,
}
impl fmt::Debug for Recovered {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Recovered")
            .field("alternatives", &self.alternatives.len())
            .field("residual", &self.residual)
            .finish()
    }
}
impl Recovered {
    /// SQL coordinates are projected through the original physical pieces.
    pub fn position<'a>(
        sql: &'a RecoveredSql,
        line: usize,
        column: usize,
        caller: &'a Path,
    ) -> (&'a Path, usize) {
        let origin = sql.positions.iter().rev().find(|origin| {
            (origin.sql_line as usize, origin.sql_column as usize) <= (line, column)
        });
        match origin {
            Some(origin) => (
                origin.source_path.as_deref().unwrap_or(caller),
                origin.source_line as usize + line - origin.sql_line as usize,
            ),
            None => (caller, sql.line as usize + line - 1),
        }
    }
    pub fn complete(&self) -> bool {
        !self.residual && (self.nonexecuting || !self.alternatives.is_empty())
    }
}
