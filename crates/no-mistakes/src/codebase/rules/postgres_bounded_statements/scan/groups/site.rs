use crate::codebase::postgres::statements::{SqlBoundFact, SqlFactSite};
use crate::codebase::postgres::SqlStatementFileFacts;
use crate::codebase::rules::postgres_findings::site_identity::{Colors, SiteIdentities};
use crate::fx::FxHashMap;
use std::path::PathBuf;

type Coordinates = (usize, usize);
type Physical = (PathBuf, usize);
pub(super) type Scope = (PathBuf, Option<super::Execution>, Physical);

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Position {
    Source(Physical, Vec<u32>, usize),
    Sql(Coordinates),
}

#[derive(Default)]
pub(super) struct Positions {
    projection: Option<usize>,
    identities: SiteIdentities<Scope>,
    occurrences: FxHashMap<(Physical, Vec<u32>), FxHashMap<Coordinates, usize>>,
}

impl Positions {
    pub(super) fn colors(self) -> Colors<Scope> {
        self.identities.finish()
    }
    pub(super) fn identify(
        &mut self,
        file: &SqlStatementFileFacts,
        bound_index: usize,
        bound: &SqlBoundFact,
        projection: Option<usize>,
        execution: Option<super::Execution>,
    ) -> (Position, Coordinates) {
        let mut fallback = bound.statement_start.unwrap_or((bound.line, bound.column));
        if file.variant_locations.is_none() && projection.is_some() {
            fallback.0 = fallback.0.saturating_sub(file.origin_line) + 1;
        }
        let position = file
            .variant_locations
            .as_ref()
            .and_then(|locations| locations.position(SqlFactSite::BoundStatement(bound_index)));
        let at = position
            .map(|position| (position.sql_line, position.sql_column))
            .unwrap_or(fallback);
        let Some(physical) = position.and_then(|position| {
            position.source_offset.map(|offset| {
                (
                    position
                        .source_path
                        .clone()
                        .unwrap_or_else(|| file.path.clone()),
                    offset,
                )
            })
        }) else {
            return (Position::Sql(at), at);
        };
        if self.projection != projection {
            self.projection = projection;
            self.occurrences.clear();
        }
        let invocation = file
            .variant_locations
            .as_ref()
            .and_then(|locations| {
                locations
                    .grouping_invocations
                    .get(&SqlFactSite::BoundStatement(bound_index))
            })
            .cloned()
            .unwrap_or_default();
        let occurrences = self
            .occurrences
            .entry((physical.clone(), invocation.clone()))
            .or_default();
        let next = occurrences.len();
        let occurrence = *occurrences.entry(at).or_insert(next);
        self.identities.record(
            (file.path.clone(), execution, physical.clone()),
            projection.unwrap_or_default(),
            (invocation.clone(), occurrence),
        );
        (Position::Source(physical, invocation, occurrence), at)
    }
}
