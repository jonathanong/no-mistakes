//! One prepared SQL parse feeds every requested statement projection.
use super::SqlStatementFileFacts;
use crate::codebase::postgres::parse::{parse_postgres_sql_lenient_with_sources, PreparedSql};
use crate::codebase::postgres::statements::value::PlaceholderPositions;
use sqlparser::ast::Statement;

/// Extract INSERT/SELECT/trigger facts from one SQL source.
pub fn extract_sql_statement_facts(sql: &str) -> SqlStatementFileFacts {
    extract_sql_statement_facts_with_bounds(sql, true)
}

/// Extract the requested SQL projections without reparsing for row bounds.
pub(crate) fn extract_sql_statement_facts_with_bounds(
    sql: &str,
    collect_bounds: bool,
) -> SqlStatementFileFacts {
    extract_sql_statement_facts_with_placeholder_positions(sql, collect_bounds, None, false)
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
        true,
    )
}

fn extract_sql_statement_facts_with_placeholder_positions(
    sql: &str,
    collect_bounds: bool,
    placeholder_positions: PlaceholderPositions<'_>,
    locations: bool,
) -> SqlStatementFileFacts {
    let prepared = PreparedSql::new(sql);
    let parsed = prepared.parse_policy();
    let parse_failed = parsed.is_err();
    match parsed {
        Ok(statements) => super::extract_from_parsed_and_sources(
            sql,
            &prepared,
            &statements,
            super::StatementSources {
                locations,
                ..Default::default()
            },
            false,
            collect_bounds,
            placeholder_positions,
        ),
        Err(_) => {
            let (located, functions) = crate::codebase::postgres::parse::partition_function_sources(
                parse_postgres_sql_lenient_with_sources(sql, prepared.normalized()),
            );
            let (statements, sources): (Vec<_>, Vec<_>) = located
                .into_iter()
                .map(|located| (located.statement, located.source))
                .unzip();
            super::extract_from_parsed_and_sources(
                sql,
                &prepared,
                &statements,
                super::StatementSources {
                    locations,
                    tokens: Some(&sources),
                    policy: super::StatementPolicySources {
                        schema: None,
                        functions: &functions,
                    },
                },
                parse_failed,
                collect_bounds,
                placeholder_positions,
            )
        }
    }
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
            locations: false,
            tokens: None,
            policy,
        },
        parse_failed,
        collect_bounds,
        placeholder_positions,
    )
}
