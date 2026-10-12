#[cfg(not(coverage))]
use super::{async_task, AsyncTask};
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::postgres::{
    extract_executed_postgres_sql_with_session, EmbeddedSqlOptions, ExecutedPostgresSource,
    TrustedSqlTag,
};
use crate::codebase::postgres::{parse_postgres_source, parse_postgres_sources, PostgresSqlSource};
#[cfg(all(not(test), not(coverage)))]
use napi_derive::napi;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(untagged)]
enum Sources {
    Single(PostgresSqlSource),
    Batch(Vec<PostgresSqlSource>),
}

pub(crate) fn parse_postgres_sql_json_impl(value: serde_json::Value) -> napi::Result<String> {
    let sources = super::options::parse_options_value::<Sources>(value)?;
    let facts = match sources {
        Sources::Single(source) => serde_json::to_string(&parse_postgres_source(&source)),
        Sources::Batch(sources) => serde_json::to_string(&parse_postgres_sources(&sources)),
    };
    // Source facts contain only JSON-safe primitives and deterministic typed collections.
    Ok(facts.expect("serializable PostgreSQL source facts"))
}

json_binding!(
    parse_postgres_sql_json,
    "parsePostgresSqlJson",
    parse_postgres_sql_json_impl,
    pure
);

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExecutedSqlRequest {
    sources: Vec<ExecutedPostgresSource>,
    import_specifier: String,
    #[serde(default)]
    executor_names: Vec<String>,
    #[serde(default)]
    executor_factory_names: Vec<String>,
    #[serde(default)]
    executor_type_names: Vec<String>,
    #[serde(default)]
    trusted_sql_tags: Vec<TrustedSqlTag>,
}

pub(crate) fn extract_executed_postgres_sql_json_impl(
    value: serde_json::Value,
) -> napi::Result<String> {
    let request = super::options::parse_options_value::<ExecutedSqlRequest>(value)?;
    let options =
        EmbeddedSqlOptions::configured(&request.import_specifier, &request.executor_names)
            .with_scoped_executors(
                &request.executor_factory_names,
                &request.executor_type_names,
            )
            .with_trusted_sql_tags(&request.trusted_sql_tags);
    let session = AnalysisSession::disabled();
    let statements =
        extract_executed_postgres_sql_with_session(&request.sources, &options, &session)
            .map_err(|error| napi::Error::from_reason(error.to_string()))?;
    serde_json::to_string(&statements).map_err(|error| napi::Error::from_reason(error.to_string()))
}

json_binding!(
    extract_executed_postgres_sql_json,
    "extractExecutedPostgresSqlJson",
    extract_executed_postgres_sql_json_impl,
    pure
);

#[cfg(test)]
mod tests;
