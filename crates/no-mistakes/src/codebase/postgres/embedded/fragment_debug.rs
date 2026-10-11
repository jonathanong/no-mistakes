use super::{EmbeddedSqlCall, EmbeddedSqlVariant, PendingRelativeScope};
use std::path::PathBuf;

/// A SQL fragment returned from a builder or appended to a
/// statement builder. These are deliberately separate from executed calls:
/// structural policies can inspect them without treating builder code as an
/// executed DML statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedSqlFragment {
    pub line: u32,
    /// `None` records a proven SQL-builder append whose argument cannot be
    /// recovered. Structural policies can then honor their fail-closed mode.
    pub sql_text: Option<String>,
    /// SQL-local positions of generated interpolation markers in `sql_text`.
    pub recovered_placeholder_positions: Vec<(u32, u32)>,
}

/// Embedded-SQL facts for one TypeScript/JavaScript file.
#[derive(Clone, PartialEq, Eq)]
pub struct EmbeddedSqlFileFacts {
    pub path: PathBuf,
    pub executor_bindings: Vec<String>,
    pub calls: Vec<EmbeddedSqlCall>,
    /// Private call identities, parallel to `calls`; public summaries stay unchanged.
    pub(crate) call_spans: Vec<(u32, u32)>,
    /// Real TypeScript comments from the same parse, excluding SQL literal text.
    pub(crate) source_comment_spans: Vec<(u32, u32)>,
    pub fragments: Vec<EmbeddedSqlFragment>,
    /// Private complete alternatives, parallel to the legacy fragment records.
    pub(crate) fragment_sites: Vec<Option<u32>>,
    pub(crate) fragment_variants: Vec<Vec<EmbeddedSqlVariant>>,
    /// Configured `executor_factory_names` this file imports from the configured module.
    pub matched_factory_names: Vec<String>,
    /// Configured `executor_type_names` this file imports from the configured module.
    pub matched_type_names: Vec<String>,
    /// Relative imports of configured names, projected after the request resolver exists.
    pub(crate) pending_relative: PendingRelativeScope,
}

// Fragment alternatives are internal provenance, so the legacy public debug
// surface remains byte-identical even when precise fragment origins exist.
impl std::fmt::Debug for EmbeddedSqlFileFacts {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EmbeddedSqlFileFacts")
            .field("path", &self.path)
            .field("executor_bindings", &self.executor_bindings)
            .field("calls", &self.calls)
            .field("call_spans", &self.call_spans)
            .field("fragments", &self.fragments)
            .field("matched_factory_names", &self.matched_factory_names)
            .field("matched_type_names", &self.matched_type_names)
            .field("pending_relative", &self.pending_relative)
            .finish()
    }
}
