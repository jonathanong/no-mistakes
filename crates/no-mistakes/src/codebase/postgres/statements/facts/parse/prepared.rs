use super::super::sources::StatementProjection;
use super::super::SqlStatementFileFacts;
use crate::codebase::postgres::conflict::ConflictChunks;
use crate::codebase::postgres::parse::{
    orphan_chunk_tokens, parse_postgres_sql_lenient_with_sources, partition_function_sources,
    PreparedSql,
};
use crate::codebase::postgres::statement_facts::SqlLocationDemand;
use crate::codebase::postgres::statements::value::PlaceholderPositions;
use sqlparser::tokenizer::TokenWithSpan;
use std::sync::Arc;

pub(super) fn extract(
    sql: &str,
    collect_bounds: bool,
    placeholder_positions: PlaceholderPositions<'_>,
    locations: SqlLocationDemand,
    projection: StatementProjection,
    collect_json_writes: bool,
) -> SqlStatementFileFacts {
    super::super::locations::work::statement_extraction();
    let prepared = PreparedSql::new(sql);
    let parsed = prepared.parse_policy();
    let parse_failed = parsed.is_err();
    match parsed {
        Ok(statements) => super::super::projection::extract(
            sql,
            &prepared,
            &statements,
            super::super::StatementSources {
                projection,
                locations,
                policy: super::super::StatementPolicySources {
                    collect_json_writes,
                    ..Default::default()
                },
                ..Default::default()
            },
            false,
            collect_bounds,
            placeholder_positions,
        ),
        Err(_) => {
            let parsed = parse_postgres_sql_lenient_with_sources(sql, prepared.normalized());
            let parse_failures: Vec<_> =
                crate::codebase::postgres::migration::recovery_failures(&parsed)
                    .into_iter()
                    .filter(|failure| failure.statement_unanalyzable)
                    .collect();
            let orphans = orphan_chunk_tokens(&parsed);
            let (mut located, functions) = partition_function_sources(parsed);
            prepared.adopt_recovered_arbiters(&mut located);
            let complete: Vec<bool> = located.iter().map(|statement| statement.complete).collect();
            let chunks: Vec<Option<Arc<[TokenWithSpan]>>> = located
                .iter()
                .map(|statement| statement.chunk_tokens.clone())
                .collect();
            let recovery = ConflictChunks {
                complete: &complete,
                chunks: &chunks,
                orphans: &orphans,
            };
            let policy_sources = super::super::sources::policy_tokens(&located);
            let (statements, sources): (Vec<_>, Vec<_>) = located
                .into_iter()
                .map(|located| (located.statement, located.source))
                .unzip();
            super::super::projection::extract(
                sql,
                &prepared,
                &statements,
                super::super::StatementSources {
                    projection,
                    locations,
                    parse_failures: &parse_failures,
                    tokens: Some(&sources),
                    recovery: Some(&recovery),
                    policy: super::super::StatementPolicySources {
                        collect_json_writes,
                        tokens: Some(&policy_sources),
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
