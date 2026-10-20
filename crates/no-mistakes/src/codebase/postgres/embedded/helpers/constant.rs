//! An exported `const` whose initializer is already SQL text.
//!
//! Importers splice it through the same hole as a helper. A tagged fragment is
//! only safe for `.append`; an untagged template would stringify the object.

use crate::codebase::postgres::embedded::walk::resolve::functions::provenance::RecoveredSql;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct ExportedConstant {
    pub(crate) sql: RecoveredSql,
    /// `sql\`...\`` statement object, not a string literal.
    pub(crate) fragment: bool,
}
