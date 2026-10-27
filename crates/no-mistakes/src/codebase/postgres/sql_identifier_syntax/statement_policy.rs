//! Application-local SQL policies borrow one request's syntax and parsed SQL facts.
use super::{helper_recovery::ApplicationKey, EmbeddedSqlOptions};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::{EmbeddedSqlFileFacts, SqlStatementFileFacts};
use std::{collections::BTreeMap, path::Path};
mod calls;
pub(crate) mod direct;
mod project;
#[cfg(test)]
mod tests;

pub(crate) struct Projection {
    pub embedded: EmbeddedSqlFileFacts,
    pub statements: Vec<SqlStatementFileFacts>,
}
pub(crate) type Projections = BTreeMap<ApplicationKey, Result<Projection, String>>;
pub(crate) fn supports(rule: &str) -> bool {
    matches!(
        rule,
        "postgres-explicit-columns" | "postgres-required-predicates"
    )
}

pub(crate) fn projection<'a>(
    facts: &'a CheckFactMap,
    path: &Path,
    profile: &EmbeddedSqlOptions,
    rule: &str,
    index: usize,
) -> anyhow::Result<&'a Projection> {
    let file = facts
        .ts
        .get(path)
        .ok_or_else(|| anyhow::anyhow!("prepared facts missing for {}", path.display()))?;
    let key = ApplicationKey::new(profile, rule, index);
    // A shared profile cannot grant another application's file membership.
    anyhow::ensure!(
        file.sql_identifier_syntax.includes_application(&key),
        "{rule}: prepared low-effort SQL application missing for {}",
        path.display()
    );
    let projections = file
        .sql_identifier_syntax
        .policy_projection
        .get_or_init(|| {
            file.sql_identifier_syntax
                .application_keys()
                .into_iter()
                .filter(|key| supports(&key.rule))
                .map(|key| {
                    let result = project::build(file, facts, path, &key);
                    (key, result.map_err(|error| error.to_string()))
                })
                .collect()
        });
    projections
        .get(&key)
        .ok_or_else(|| anyhow::anyhow!("unsupported low SQL policy {rule}"))?
        .as_ref()
        .map_err(|error| anyhow::anyhow!(error.clone()))
}
