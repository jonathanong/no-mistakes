use crate::codebase::postgres::embedded::{helpers, relative::PendingRelativeCall};
use crate::codebase::postgres::{
    EmbeddedSqlCall, EmbeddedSqlFragment, EmbeddedSqlVariant, UndeclaredExecutorType,
};
use std::collections::BTreeMap;

pub(crate) struct CollectedCalls {
    #[cfg(feature = "test-instrumentation")]
    pub(crate) lock_summary_passes: usize,
    pub(crate) lock_helper_summaries:
        crate::fx::FxHashMap<String, super::super::lock_helpers::Summary>,
    pub(crate) imported_lock_helpers: Vec<super::super::lock_helpers::Invocation>,
    pub(crate) undeclared_executor_types: Vec<UndeclaredExecutorType>,
    pub(crate) calls: Vec<EmbeddedSqlCall>,
    pub(crate) call_spans: Vec<(u32, u32)>,
    pub(crate) exported_helpers: crate::fx::FxHashMap<String, helpers::HelperSummary>,
    pub(crate) exported_programs:
        crate::fx::FxHashMap<String, std::sync::Arc<helpers::program::Program>>,
    pub(crate) exported_constants: crate::fx::FxHashMap<String, helpers::ExportedConstant>,
    pub(crate) pending_imported: Vec<helpers::PendingImportedSql>,
    pub(crate) pending_spans: BTreeMap<u32, (u32, u32)>,
    pub(crate) fragments: Vec<EmbeddedSqlFragment>,
    pub(crate) fragment_sites: Vec<Option<u32>>,
    pub(crate) fragment_variants: Vec<Vec<EmbeddedSqlVariant>>,
    pub(crate) call_fragments:
        crate::fx::FxHashMap<(u32, u32), Vec<crate::codebase::postgres::embedded::CallSqlFragment>>,
    pub(crate) param_append_fragments: Vec<crate::codebase::postgres::embedded::CallSqlFragment>,
    pub(crate) pending_calls: Vec<PendingRelativeCall>,
    pub(crate) diagnostic_member_sequences: Vec<u32>,
    pub(crate) confirmed_order: Vec<u32>,
    pub(crate) unproven_member_query_spans: Vec<(u32, u32)>,
}

pub(super) fn collect_functions<'a>(
    program: &oxc_ast::ast::Program<'a>,
    source: &'a str,
    options: &crate::codebase::postgres::embedded::EmbeddedSqlOptions,
    head_calls_only: bool,
) -> super::super::resolve::LocalFunctions {
    if head_calls_only {
        super::super::resolve::LocalFunctions::collect_heads_with_raw(
            program,
            &options.trusted_sql_tags,
            &options.raw_sql_tags,
        )
    } else {
        super::super::resolve::LocalFunctions::collect_with_raw(
            program,
            source,
            &options.trusted_sql_tags,
            &options.raw_sql_tags,
        )
    }
}
