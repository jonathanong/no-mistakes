//! One prepared SQL parse feeds every requested statement projection.
use super::sources::StatementProjection;
use super::SqlStatementFileFacts;
use crate::codebase::postgres::parse::PreparedSql;
use crate::codebase::postgres::statement_facts::SqlLocationDemand;
use crate::codebase::postgres::statements::value::PlaceholderPositions;
use sqlparser::ast::Statement;

mod prepared;
use prepared::extract as extract_sql_statement_facts_with_placeholder_positions;

/// Extract INSERT/SELECT/trigger facts from one SQL source.
pub fn extract_sql_statement_facts(sql: &str) -> SqlStatementFileFacts {
    extract_sql_statement_facts_with_placeholder_positions(
        sql,
        true,
        None,
        SqlLocationDemand::None,
        StatementProjection::Complete,
        true,
    )
}

/// Extract the requested SQL projections without reparsing for row bounds.
pub(crate) fn extract_sql_statement_facts_with_bounds(
    sql: &str,
    collect_bounds: bool,
) -> SqlStatementFileFacts {
    extract_sql_statement_facts_with_placeholder_positions(
        sql,
        collect_bounds,
        None,
        SqlLocationDemand::None,
        StatementProjection::Complete,
        false,
    )
}

/// Extract facts while identifying the SQL-local positions of recovered interpolation markers.
pub(crate) fn extract_sql_statement_facts_with_recovered_placeholders(
    sql: &str,
    collect_bounds: bool,
    recovered_placeholder_positions: &[(u32, u32)],
) -> SqlStatementFileFacts {
    extract_sql_statement_facts_with_placeholder_positions(
        sql,
        collect_bounds,
        Some(recovered_placeholder_positions),
        SqlLocationDemand::None,
        StatementProjection::Complete,
        false,
    )
}

pub(crate) fn extract_sql_variant_statement_facts(
    sql: &str,
    collect_bounds: bool,
    positions: &[(u32, u32)],
) -> SqlStatementFileFacts {
    extract_sql_statement_facts_with_placeholder_positions(
        sql,
        collect_bounds,
        Some(positions),
        SqlLocationDemand::Full,
        StatementProjection::Complete,
        false,
    )
}

pub(crate) fn extract_sql_statement_facts_with_location_demand(
    sql: &str,
    collect_bounds: bool,
    positions: &[(u32, u32)],
    demand: SqlLocationDemand,
) -> SqlStatementFileFacts {
    extract_sql_statement_facts_with_placeholder_positions(
        sql,
        collect_bounds,
        Some(positions),
        demand,
        StatementProjection::Complete,
        false,
    )
}

pub(crate) fn extract_sql_statement_facts_with_json_writes(
    sql: &str,
    collect_bounds: bool,
    positions: PlaceholderPositions<'_>,
    demand: SqlLocationDemand,
    collect_json_writes: bool,
) -> SqlStatementFileFacts {
    extract_sql_statement_facts_with_placeholder_positions(
        sql,
        collect_bounds,
        positions,
        demand,
        StatementProjection::Complete,
        collect_json_writes,
    )
}

/// Prepare only the fields consumed by the sole conflict-ordering policy.
pub(crate) fn extract_sql_conflict_policy_facts(
    sql: &str,
    positions: &[(u32, u32)],
) -> SqlStatementFileFacts {
    extract_sql_statement_facts_with_placeholder_positions(
        sql,
        false,
        Some(positions),
        SqlLocationDemand::Conflicts,
        StatementProjection::ConflictPolicy,
        false,
    )
}

pub(crate) fn extract_from_parsed_with_recovered_placeholders(
    sql: &str,
    prepared: &PreparedSql<'_>,
    statements: &[Statement],
    parse_failed: bool,
    collect_bounds: bool,
    placeholder_positions: PlaceholderPositions<'_>,
    policy: super::StatementPolicySources<'_>,
) -> SqlStatementFileFacts {
    super::extract_from_parsed_and_sources(
        sql,
        prepared,
        statements,
        super::StatementSources {
            locations: SqlLocationDemand::None,
            tokens: None,
            recovery: None,
            policy,
            ..Default::default()
        },
        parse_failed,
        collect_bounds,
        placeholder_positions,
    )
}

#[cfg(test)]
mod location_demand_tests;

#[cfg(test)]
mod mapped_demand_tests;

#[cfg(test)]
mod policy_projection_tests;

#[cfg(test)]
mod json_write_demand_tests;
