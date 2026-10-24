//! An exported `const` whose initializer is already SQL text.
//!
//! Importers splice strings through the same hole as a helper. Tagged statement
//! objects remain unknown across imports because any importer can mutate them.

use crate::codebase::postgres::embedded::walk::resolve::functions::provenance::RecoveredSql;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct ExportedConstant {
    pub(crate) sql: RecoveredSql,
    /// `sql\`...\`` statement object, not a string literal.
    pub(crate) fragment: bool,
}
