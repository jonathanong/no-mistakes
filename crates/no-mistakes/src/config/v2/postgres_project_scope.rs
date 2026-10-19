use anyhow::{bail, Result};
use serde_yaml::Value;

use super::schema::{NoMistakesConfig, ProjectPostgresTrustedSqlTag};

mod effects;
mod inherit;
mod narrow;
mod scope;
#[cfg(test)]
mod tests;
mod yaml;

/// Copied project PostgreSQL scope. Lists that rules compare are already in the
/// basis each consumer uses (project-root-relative or repository-relative).
struct Scope {
    name: String,
    import_specifier: String,
    factory_names: Vec<String>,
    type_names: Vec<String>,
    trusted_sql_tags: Vec<ProjectPostgresTrustedSqlTag>,
    schema_catalog_path: String,
    unanalyzable_sql: String,
    sql_include: Vec<String>,
    raw_includes: Vec<String>,
    option_includes: Vec<String>,
    roots: Vec<Option<String>>,
}

/// Inherit `projects.<name>.postgres` into postgres rules and `effects.postgres`.
///
/// The first successful call sets `postgres_project_scope_materialized` when it
/// writes `sqlIncludeLocked` or expands effect selectors. A second call sees
/// that flag and returns without duplicating selectors. User-supplied
/// `sqlIncludeLocked` is rejected while the flag is still unset.
pub(crate) fn materialize(config: &mut NoMistakesConfig) -> Result<()> {
    if config.postgres_project_scope_materialized {
        return Ok(());
    }
    reject_user_sql_include_locked(config)?;
    let inherited = inherit::apply(config)?;
    let expanded = effects::apply(config)?;
    if inherited || expanded {
        config.postgres_project_scope_materialized = true;
    }
    Ok(())
}

fn reject_user_sql_include_locked(config: &NoMistakesConfig) -> Result<()> {
    for (index, rule) in config.rules.iter().enumerate() {
        let locked = rule
            .options
            .as_mapping()
            .and_then(|map| map.get(Value::String("sqlIncludeLocked".to_string())));
        if locked.is_some() {
            bail!(
                "rules[{index}] options.sqlIncludeLocked is internal and cannot be set; remove it"
            );
        }
    }
    Ok(())
}
