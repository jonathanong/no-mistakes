//! Additive diagnostics over the same extraction, without treating omissions as SQL.
use super::{
    EmbeddedSqlOptions, ExecutedPostgresResolvedImport, ExecutedPostgresSource, ExecutedPostgresSql,
};
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::postgres::UndeclaredExecutorTypeSummary;
use anyhow::Result;
use serde::Serialize;

/// Executed statements plus missing-type and unresolved-receiver diagnostics.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutedPostgresSqlReport {
    pub statements: Vec<ExecutedPostgresSql>,
    pub undeclared_executor_types: Vec<UndeclaredExecutorTypeSummary>,
    /// Configured imported types whose receiver shapes have no executor proof.
    pub unresolved_executor_receivers: Vec<UndeclaredExecutorTypeSummary>,
}

/// Preserve extraction's statements while exposing diagnostic counts. `Some`
/// selects supplied-import resolution, including when the edge list is empty.
pub fn extract_executed_postgres_sql_report(
    sources: &[ExecutedPostgresSource],
    options: &EmbeddedSqlOptions,
    imports: Option<&[ExecutedPostgresResolvedImport]>,
) -> Result<ExecutedPostgresSqlReport> {
    let session = AnalysisSession::disabled();
    match imports {
        Some(imports) => super::extract_executed_postgres_sql_with_imports_and_session(
            sources, options, imports, &session,
        ),
        None => super::extract_executed_postgres_sql_with_session(sources, options, &session),
    }
}
