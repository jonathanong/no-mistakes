use crate::codebase::postgres::embedded::{helpers, relative::PendingRelativeCall};
use crate::codebase::postgres::{
    EmbeddedSqlCall, EmbeddedSqlFragment, EmbeddedSqlVariant, UndeclaredExecutorType,
};
use std::collections::BTreeMap;

pub(crate) struct CollectedCalls {
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
    pub(crate) pending_calls: Vec<PendingRelativeCall>,
    pub(crate) diagnostic_member_sequences: Vec<u32>,
    pub(crate) confirmed_order: Vec<u32>,
    pub(crate) unproven_member_query_spans: Vec<(u32, u32)>,
}
