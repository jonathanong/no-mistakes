use sqlparser::tokenizer::TokenWithSpan;
use std::sync::Arc;

#[derive(Default)]
pub(crate) struct StatementPolicySources<'a> {
    pub(crate) collect_json_writes: bool,
    pub(crate) tokens: Option<&'a [Option<Arc<[TokenWithSpan]>>]>,
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
    pub(super) parse_failures: &'a [crate::codebase::postgres::SqlUnanalyzableStatement],
    pub(super) locations: crate::codebase::postgres::statement_facts::SqlLocationDemand,
    pub(super) tokens: Option<&'a [Option<Arc<[TokenWithSpan]>>]>,
    // `recovery` is parallel to statements after a failed strict parse.
    pub(super) policy: StatementPolicySources<'a>,
    pub(super) recovery: Option<&'a crate::codebase::postgres::conflict::ConflictChunks<'a>>,
}

impl<'a> StatementSources<'a> {
    pub(super) fn location_tokens(
        &self,
        index: usize,
        fallback: &'a [TokenWithSpan],
    ) -> &'a [TokenWithSpan] {
        self.tokens
            .or(self.policy.tokens)
            .and_then(|tokens| tokens.get(index))
            .and_then(Option::as_deref)
            .unwrap_or(fallback)
    }
}

pub(super) fn policy_tokens(
    located: &[crate::codebase::postgres::parse::LocatedStatement],
) -> Vec<Option<Arc<[TokenWithSpan]>>> {
    located
        .iter()
        .map(|located| {
            located
                .source
                .clone()
                .or_else(|| located.chunk_tokens.clone())
        })
        .collect()
}
