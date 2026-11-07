use crate::codebase::postgres::statements::{SqlBoundFact, SqlFactSite};
use crate::codebase::postgres::SqlStatementFileFacts;
use crate::fx::FxHashMap;
use std::path::PathBuf;

type Coordinates = (usize, usize);
type Physical = (PathBuf, usize);

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Position {
    Source(Physical, usize),
    Sql(Coordinates),
}

#[derive(Default)]
pub(super) struct Positions {
    projection: Option<usize>,
    occurrences: FxHashMap<Physical, FxHashMap<Coordinates, usize>>,
}

impl Positions {
    pub(super) fn identify(
        &mut self,
        file: &SqlStatementFileFacts,
        bound_index: usize,
        bound: &SqlBoundFact,
        projection: Option<usize>,
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
        let occurrences = self.occurrences.entry(physical.clone()).or_default();
        let next = occurrences.len();
        let occurrence = *occurrences.entry(at).or_insert(next);
        (Position::Source(physical, occurrence), at)
    }
}
