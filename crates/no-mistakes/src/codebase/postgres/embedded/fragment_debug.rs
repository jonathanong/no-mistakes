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
    /// Summaries of this file's helpers, collected during its single AST pass.
    pub(crate) exported_helpers: crate::fx::FxHashMap<String, super::helpers::HelperSummary>,
    /// Lowered bodies of the same helpers, run per call site by the projection.
    pub(crate) exported_programs:
        crate::fx::FxHashMap<String, std::sync::Arc<super::helpers::program::Program>>,
    pub(crate) exported_constants: crate::fx::FxHashMap<String, super::helpers::ExportedConstant>,
    /// Executor SQL that is complete once imported helpers resolve.
    pub(crate) pending_imported: Vec<super::helpers::PendingImportedSql>,
    /// Caller lines of the imported-helper calls completed into a call, by span.
    pub(crate) imported_call_lines: crate::fx::FxHashMap<(u32, u32), Vec<u32>>,
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
    /// SQL-looking member `.query` calls with no executor proof, by source span.
    pub(crate) unproven_member_query_spans: Vec<(u32, u32)>,
    /// PostgreSQL rules that share this options profile. Empty means the
    /// extract has no check rule, so suppression does not apply.
    pub(crate) postgres_rule_ids: Vec<String>,
    /// When false, a later relative projection must not add member spans.
    /// Every configured rule for this profile excluded the file.
    pub(crate) record_unproven_spans: bool,
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

impl EmbeddedSqlFileFacts {
    /// Replace the unrecovered call at `span` with its completed form. Only a
    /// call still left `Dynamic` by the walk is replaced.
    pub(crate) fn apply_imported_sql(
        &mut self,
        span: (u32, u32),
        completed: EmbeddedSqlCall,
        lines: Vec<u32>,
    ) {
        let Some(index) = self
            .call_spans
            .iter()
            .position(|candidate| *candidate == span)
        else {
            return;
        };
        let call = &self.calls[index];
        if call.kind == super::EmbeddedSqlKind::Dynamic && call.variants.is_empty() {
            self.calls[index] = completed;
            self.imported_call_lines.insert(span, lines);
        }
    }
}
