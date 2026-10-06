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
    extract_sql_statement_facts_with_placeholder_positions(sql, collect_bounds, None)
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
    )
}

fn extract_sql_statement_facts_with_placeholder_positions(
    sql: &str,
    collect_bounds: bool,
    placeholder_positions: PlaceholderPositions<'_>,
) -> SqlStatementFileFacts {
    let prepared = PreparedSql::new(sql);
    let parsed = prepared.parse();
    let parse_failed = parsed.is_err();
    match parsed {
        Ok(statements) => super::extract_from_parsed_and_sources(
            sql,
            &prepared,
            &statements,
            super::StatementSources::default(),
            false,
            collect_bounds,
            placeholder_positions,
        ),
        Err(_) => {
            let (statements, sources): (Vec<_>, Vec<_>) =
                parse_postgres_sql_lenient_with_sources(sql, prepared.normalized())
                    .into_iter()
                    .map(|located| (located.statement, located.source))
                    .unzip();
            super::extract_from_parsed_and_sources(
                sql,
                &prepared,
                &statements,
                super::StatementSources {
                    tokens: Some(&sources),
                    policy: None,
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
    schema_policy: Option<&crate::codebase::postgres::SqlSchemaFileFacts>,
) -> SqlStatementFileFacts {
    super::extract_from_parsed_and_sources(
        sql,
        prepared,
        statements,
        super::StatementSources {
            tokens: None,
            policy: schema_policy,
        },
        parse_failed,
        collect_bounds,
        placeholder_positions,
    )
}
