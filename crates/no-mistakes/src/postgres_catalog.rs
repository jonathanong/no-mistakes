//! Independent, read-only PostgreSQL schema catalog generation.
mod connection;
mod sql;

use anyhow::{bail, Context, Result};
pub(crate) use connection::connection_environment;
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Which facts the generated catalog carries, stated in its `coverage` field.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize, clap::ValueEnum)]
#[serde(rename_all = "camelCase")]
pub enum PostgresCatalogCoverage {
    /// Every fact the catalog model holds. Every catalog rule accepts it.
    #[default]
    Complete,
    /// Only the facts conflict and lock ordering need; other catalog rules reject it.
    Ordering,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PostgresCatalogOptions {
    /// Name of the environment variable containing a PostgreSQL connection URL.
    pub connection_env: String,
    /// Exact PostgreSQL schema name (not an SQL expression).
    pub schema: String,
    /// `complete` (the default) or `ordering`.
    #[serde(default)]
    pub coverage: PostgresCatalogCoverage,
    /// Schemas whose accessibility and complete relation-name sets are needed for search_path.
    #[serde(default)]
    pub search_path_schemas: Vec<String>,
}

/// Observe committed database metadata in one repeatable-read, read-only transaction.
/// Connection secrets are passed only through the child environment and never diagnostics.
pub fn generate(options: &PostgresCatalogOptions) -> Result<serde_json::Value> {
    if options.schema.is_empty() || options.schema.contains('\0') {
        bail!("schema must be a non-empty PostgreSQL schema name");
    }
    if options
        .search_path_schemas
        .iter()
        .any(|schema| schema.is_empty() || schema.contains('\0'))
    {
        bail!("searchPathSchemas must contain non-empty PostgreSQL schema names");
    }
    if options.connection_env.is_empty() || options.connection_env.contains(['=', '\0']) {
        bail!("connectionEnv must name an environment variable");
    }
    let connection = std::env::var(&options.connection_env).map_err(|_| {
        anyhow::anyhow!("connection environment variable is missing or not Unicode")
    })?;
    if connection.is_empty() {
        bail!("connection environment variable is empty");
    }
    let mut command = Command::new("psql");
    connection_environment(&connection, &mut command)?;
    command.args([
        "--no-psqlrc",
        "--quiet",
        "--tuples-only",
        "--no-align",
        "--no-password",
        "--set",
        "ON_ERROR_STOP=1",
        "--command",
        &sql::catalog_query_with_search_path(
            &options.schema,
            options.coverage,
            &options.search_path_schemas,
        ),
    ]);
    let output = crate::invocation::command_output(&mut command)
        .context("failed to execute psql; install PostgreSQL client tools")?;
    if !output.status.success() {
        // libpq errors may include the connection string. Never echo stderr.
        bail!("PostgreSQL catalog query failed; verify connection, permissions, and server compatibility");
    }
    let mut catalog: serde_json::Value = serde_json::from_slice(&output.stdout)
        .context("PostgreSQL did not return a catalog; verify that the requested schema exists")?;
    // jsonb orders keys by length; sort them so committed catalogs read and diff naturally.
    catalog.sort_all_objects();
    Ok(catalog)
}

#[cfg(test)]
mod tests;
