use crate::lock_inventory::{tokens::Segments, Collector};
use sqlparser::tokenizer::TokenWithSpan;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct StatementPolicySources<'a> {
    pub bind_values: super::super::bounds::BindValues,
    pub collect_lock_inventory: bool,
    pub collect_json_writes: bool,
    pub collect_duplicate_keyed_lookups: bool,
    pub collect_grouping: bool,
    pub grouping_complete: Option<&'a [bool]>,
    pub collect_index_probes: bool,
    pub collect_body_bounds: bool,
    pub share_lock_writes: bool,
    pub redundant_prelocks: bool,
    pub tokens: Option<&'a [Option<Arc<[TokenWithSpan]>>]>,
    pub schema: Option<&'a crate::SqlSchemaFileFacts>,
    pub functions: &'a [crate::SqlFunctionCallFact],
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
    pub(super) parse_failures: &'a [crate::SqlUnanalyzableStatement],
    pub(super) locations: crate::statement_facts::SqlLocationDemand,
    pub(super) tokens: Option<&'a [Option<Arc<[TokenWithSpan]>>]>,
    // `recovery` is parallel to statements after a failed strict parse.
    pub(super) policy: StatementPolicySources<'a>,
    pub(super) recovery: Option<&'a crate::conflict::ConflictChunks<'a>>,
}

impl StatementSources<'_> {
    pub(super) fn locks(
        &self,
        statements: &[sqlparser::ast::Statement],
        positions: super::super::value::PlaceholderPositions<'_>,
        prepared: &super::PreparedSql<'_>,
        inventory: Option<&Collector<'_>>,
    ) -> super::super::lock_writes::LockWriteFacts {
        if matches!(
            self.projection,
            StatementProjection::LockScope | StatementProjection::BodyBoundsLockScope
        ) {
            let supplemental = (self.policy.share_lock_writes && inventory.is_none())
                .then(|| Segments::new(prepared.tokens()));
            let modes = self
                .policy
                .share_lock_writes
                .then(|| inventory.map(Collector::segments).or(supplemental.as_ref()))
                .flatten();
            super::super::lock_writes::collect::<true>(
                statements,
                positions,
                self.policy.redundant_prelocks,
                modes,
                prepared.tokens(),
            )
        } else {
            super::super::lock_writes::collect::<false>(statements, positions, false, None, &[])
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
    located: &[crate::parse::LocatedStatement],
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
    pub(super) fn lock_inventory<'sql>(
        &self,
        prepared: &'sql super::PreparedSql<'_>,
    ) -> Option<crate::lock_inventory::Collector<'sql>> {
        self.collect_lock_inventory
            .then(|| crate::lock_inventory::Collector::new(prepared))
    }

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
