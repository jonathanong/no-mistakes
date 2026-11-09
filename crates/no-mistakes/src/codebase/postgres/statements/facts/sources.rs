use sqlparser::tokenizer::TokenWithSpan;
use std::sync::Arc;

#[derive(Clone, Default)]
pub(crate) struct StatementPolicySources<'a> {
    pub(crate) bind_values: super::super::bounds::BindValues,
    pub(crate) collect_json_writes: bool,
    pub(crate) collect_grouping: bool,
    pub(crate) grouping_complete: Option<&'a [bool]>,
    pub(crate) collect_index_probes: bool,
    pub(crate) collect_body_bounds: bool,
    pub(crate) tokens: Option<&'a [Option<Arc<[TokenWithSpan]>>]>,
    pub(crate) schema: Option<&'a crate::codebase::postgres::SqlSchemaFileFacts>,
    pub(crate) functions: &'a [crate::codebase::postgres::SqlFunctionCallFact],
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum StatementProjection {
    #[default]
    Complete,
    ConflictPolicy,
    LockScope,
    BodyBounds,
    BodyBoundsLockScope,
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

impl StatementProjection {
    pub(super) fn locks(
        self,
        statements: &[sqlparser::ast::Statement],
        positions: super::super::value::PlaceholderPositions<'_>,
    ) -> super::super::lock_writes::LockWriteFacts {
        if matches!(
            self,
            StatementProjection::LockScope | StatementProjection::BodyBoundsLockScope
        ) {
            super::super::lock_writes::collect::<true>(statements, positions)
        } else {
            super::super::lock_writes::collect::<false>(statements, positions)
        }
    }
}

impl<'a> StatementSources<'a> {
    pub(super) fn grouping_requested(&self, index: usize) -> bool {
        self.policy.collect_grouping
            && self
                .recovery
                .map(|recovery| recovery.complete)
                .or(self.policy.grouping_complete)
                .is_none_or(|complete| complete.get(index) == Some(&true))
    }

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

impl StatementPolicySources<'_> {
    pub(super) fn scope(
        &self,
        index: &super::super::TableTokenIndex,
        statement: &sqlparser::ast::Statement,
    ) -> super::super::bounds::Scope {
        use sqlparser::ast::Spanned;
        super::super::bounds::Scope::with_table_tokens(index.cursor_at(statement.span().start))
            .with_bodies(self.collect_body_bounds)
            .with_bind_values(self.bind_values.clone())
    }
}
