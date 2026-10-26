//! Sole-policy projection; the existing complete-facts entrypoints retain their contract.
use super::sources::StatementProjection;
use super::{locations, SqlStatementFileFacts, StatementSources};
use crate::codebase::postgres::parse::PreparedSql;
use crate::codebase::postgres::statement_facts::SqlLocationDemand;
use crate::codebase::postgres::statements::value::PlaceholderPositions;
use sqlparser::ast::Statement;

pub(super) fn extract(
    sql: &str,
    prepared_sql: &PreparedSql<'_>,
    statements: &[Statement],
    sources: StatementSources<'_>,
    parse_failed: bool,
    collect_bounds: bool,
    placeholder_positions: PlaceholderPositions<'_>,
) -> SqlStatementFileFacts {
    if sources.projection == StatementProjection::Complete {
        return super::extract_from_parsed_and_sources(
            sql,
            prepared_sql,
            statements,
            sources,
            parse_failed,
            collect_bounds,
            placeholder_positions,
        );
    }
    let mut facts = SqlStatementFileFacts {
        parse_failed,
        parse_failures: sources.parse_failures.to_vec(),
        ..Default::default()
    };
    // Conflict analysis borrows the same prepared AST, bind positions and recovery provenance.
    facts.variant_locations = Some(locations::finish(
        locations::Locations::new(&[], SqlLocationDemand::Conflicts),
        &facts,
        prepared_sql.tokens(),
        sql,
        statements,
        placeholder_positions.unwrap_or_default(),
        sources.recovery,
    ));
    facts
}
