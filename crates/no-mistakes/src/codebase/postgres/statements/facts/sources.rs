use sqlparser::tokenizer::TokenWithSpan;
use std::sync::Arc;

#[derive(Default)]
pub(crate) struct StatementPolicySources<'a> {
    pub(crate) schema: Option<&'a crate::codebase::postgres::SqlSchemaFileFacts>,
    pub(crate) functions: &'a [crate::codebase::postgres::SqlFunctionCallFact],
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum StatementProjection {
    #[default]
    Complete,
    ConflictPolicy,
}

#[derive(Default)]
pub(super) struct StatementSources<'a> {
    pub(super) projection: StatementProjection,
    pub(super) locations: crate::codebase::postgres::statement_facts::SqlLocationDemand,
    pub(super) tokens: Option<&'a [Option<Arc<[TokenWithSpan]>>]>,
    // `recovery` is parallel to statements after a failed strict parse.
    pub(super) policy: StatementPolicySources<'a>,
    pub(super) recovery: Option<&'a crate::codebase::postgres::conflict::ConflictChunks<'a>>,
}
