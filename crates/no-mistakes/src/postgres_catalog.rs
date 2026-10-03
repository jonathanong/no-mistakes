//! Independent, read-only PostgreSQL ordering catalog generation.
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PostgresCatalogOptions {
    /// Name of the environment variable containing a PostgreSQL connection URL.
    pub connection_env: String,
    /// Exact PostgreSQL schema name (not an SQL expression).
    pub schema: String,
}

/// Observe committed database metadata in one repeatable-read, read-only transaction.
/// Connection secrets are passed only through the child environment and never diagnostics.
pub fn generate(options: &PostgresCatalogOptions) -> Result<serde_json::Value> {
    if options.schema.is_empty() || options.schema.contains('\0') {
        bail!("schema must be a non-empty PostgreSQL schema name");
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
    let literal = format!(
        "E'{}'",
        options.schema.replace('\\', "\\\\").replace('\'', "''")
    );
    let sql = include_str!("postgres_catalog/ordering.sql").replace("__SCHEMA__", &literal);
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
        &sql,
    ]);
    let output = crate::invocation::command_output(&mut command)
        .context("failed to execute psql; install PostgreSQL client tools")?;
    if !output.status.success() {
        // libpq errors may include the connection string. Never echo stderr.
        bail!("PostgreSQL catalog query failed; verify connection, permissions, and server compatibility");
    }
    let catalog: serde_json::Value = serde_json::from_slice(&output.stdout)
        .context("PostgreSQL did not return a catalog; verify that the requested schema exists")?;
    Ok(catalog)
}

#[cfg(test)]
mod tests;

fn connection_environment(raw: &str, command: &mut Command) -> Result<()> {
    let url =
        url::Url::parse(raw).map_err(|_| anyhow::anyhow!("connection must be a PostgreSQL URL"))?;
    if !matches!(url.scheme(), "postgres" | "postgresql") || url.fragment().is_some() {
        bail!("connection must be a PostgreSQL URL without a fragment");
    }
    // A service file or stale host address must not override the selected URL.
    command
        .env_remove("PGSERVICE")
        .env_remove("PGSERVICEFILE")
        .env_remove("PGHOSTADDR");
    let decode = |value: &str| {
        percent_encoding::percent_decode_str(value)
            .decode_utf8()
            .map(|value| value.into_owned())
            .map_err(|_| anyhow::anyhow!("connection URL contains invalid UTF-8"))
    };
    if let Some(host) = url.host_str() {
        command.env("PGHOST", host.trim_start_matches('[').trim_end_matches(']'));
    }
    if let Some(port) = url.port() {
        command.env("PGPORT", port.to_string());
    }
    if !url.username().is_empty() {
        command.env("PGUSER", decode(url.username())?);
    }
    if let Some(password) = url.password() {
        command.env("PGPASSWORD", decode(password)?);
    }
    let database = decode(url.path().trim_start_matches('/'))?;
    if !database.is_empty() {
        command.env("PGDATABASE", database);
    }
    for (key, value) in url.query_pairs() {
        let variable = match key.as_ref() {
            "host" => "PGHOST",
            "port" => "PGPORT",
            "user" => "PGUSER",
            "dbname" => "PGDATABASE",
            "sslmode" => "PGSSLMODE",
            "sslrootcert" => "PGSSLROOTCERT",
            "sslcert" => "PGSSLCERT",
            "sslkey" => "PGSSLKEY",
            "connect_timeout" => "PGCONNECT_TIMEOUT",
            _ => bail!(
                "unsupported PostgreSQL URL parameter; use libpq PG* environment configuration"
            ),
        };
        command.env(variable, value.as_ref());
    }
    Ok(())
}
