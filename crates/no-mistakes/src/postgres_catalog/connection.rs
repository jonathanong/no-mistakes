use anyhow::{bail, Result};
use std::process::Command;

pub(super) fn connection_environment(raw: &str, command: &mut Command) -> Result<()> {
    let url =
        url::Url::parse(raw).map_err(|_| anyhow::anyhow!("connection must be a PostgreSQL URL"))?;
    if !matches!(url.scheme(), "postgres" | "postgresql") || url.fragment().is_some() {
        bail!("connection must be a PostgreSQL URL without a fragment");
    }
    // A service file or stale host address must not override the selected URL.
    command
        .env_remove("PGSERVICE")
        .env_remove("PGSERVICEFILE")
        .env_remove("PGHOSTADDR")
        .env("PGCONNECT_TIMEOUT", "10");
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
