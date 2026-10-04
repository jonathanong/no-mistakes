#[cfg(not(coverage))]
use super::{async_task, AsyncTask};
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

#[cfg(test)]
mod tests;
