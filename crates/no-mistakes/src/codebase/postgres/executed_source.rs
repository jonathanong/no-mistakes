use super::{
    extract_embedded_sql_from_program, leading_query_annotation, EmbeddedSqlKind,
    EmbeddedSqlOptions,
};
use crate::codebase::analysis_session::AnalysisSession;
use anyhow::Result;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

/// Supplied TypeScript or JavaScript source; no repository read is performed.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutedPostgresSource {
    pub file_name: String,
    pub source: String,
}

/// One executor call or one complete alternative of a conditional executor call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutedPostgresSql {
    pub file_name: String,
    pub line: u32,
    pub annotation: Option<String>,
    pub sql_text: Option<String>,
    /// `dynamic` may expose only a verified prefix. Only complete SQL is safe
    /// for downstream statement classification.
    pub complete: bool,
    pub kind: EmbeddedSqlKind,
}

/// Reuse the embedded-SQL executor analysis for supplied source texts.
/// Ordering follows the supplied files, call sites, then concrete alternatives.
pub fn extract_executed_postgres_sql(
    sources: &[ExecutedPostgresSource],
    options: &EmbeddedSqlOptions,
) -> Result<Vec<ExecutedPostgresSql>> {
    extract_executed_postgres_sql_with_session(sources, options, &AnalysisSession::disabled())
}

pub(crate) fn extract_executed_postgres_sql_with_session(
    sources: &[ExecutedPostgresSource],
    options: &EmbeddedSqlOptions,
    session: &AnalysisSession,
) -> Result<Vec<ExecutedPostgresSql>> {
    let batches = sources
        .par_iter()
        .map(|source| extract_source(source, options, session))
        .collect::<Result<Vec<_>>>()?;
    Ok(batches.into_iter().flatten().collect())
}

fn extract_source(
    source: &ExecutedPostgresSource,
    options: &EmbeddedSqlOptions,
    session: &AnalysisSession,
) -> Result<Vec<ExecutedPostgresSql>> {
    let path = Path::new(&source.file_name);
    session.with_program(
        path,
        &Arc::from(source.source.as_str()),
        |program, source_text| {
            let facts = extract_embedded_sql_from_program(path, program, source_text, options);
            project_calls(&source.file_name, facts.calls)
        },
    )
}

fn project_calls(file_name: &str, calls: Vec<super::EmbeddedSqlCall>) -> Vec<ExecutedPostgresSql> {
    let mut statements = Vec::new();
    for call in calls {
        if call.variants.is_empty() {
            statements.push(statement(
                file_name,
                call.line,
                call.sql_text,
                call.kind != EmbeddedSqlKind::Dynamic
                    && call.recovered_placeholder_positions.is_empty(),
                call.kind,
            ));
        } else {
            for variant in call.variants {
                statements.push(statement(
                    file_name,
                    call.line,
                    Some(variant.sql_text),
                    variant.recovered_placeholder_positions.is_empty(),
                    EmbeddedSqlKind::Composed,
                ));
            }
        }
    }
    statements
}

fn statement(
    file_name: &str,
    line: u32,
    sql_text: Option<String>,
    complete: bool,
    kind: EmbeddedSqlKind,
) -> ExecutedPostgresSql {
    let annotation = sql_text
        .as_deref()
        .and_then(leading_query_annotation)
        .map(str::to_owned);
    ExecutedPostgresSql {
        file_name: file_name.to_string(),
        line,
        annotation,
        sql_text,
        complete,
        kind,
    }
}
