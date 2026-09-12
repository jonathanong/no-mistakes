use super::EmbeddedSqlOptions;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Filesystem rules whose configured executor projections must be declared at
/// the request boundary before TS/JS fact collection begins.
#[doc(hidden)]
pub const PREPARED_EMBEDDED_SQL_RULE_IDS: &[&str] =
    &["postgres-conflict-ordering", "postgres-lock-ordering"];

#[doc(hidden)]
pub const SCHEMA_CATALOG_RULE_IDS: &[&str] = PREPARED_EMBEDDED_SQL_RULE_IDS;

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct EmbeddedSqlRuleOptions {
    import_specifier: String,
    executor_names: Vec<String>,
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct CatalogRuleOptions {
    schema_catalog_path: String,
}

/// Resolve every distinct embedded-SQL projection requested by the supplied
/// configured rules. Unknown rule-specific options are deliberately ignored;
/// their owning rule validates the complete option object.
#[inline(never)]
pub(crate) fn configured_embedded_sql_options(
    config: &NoMistakesConfig,
    rule_ids: &[&str],
) -> Result<Vec<EmbeddedSqlOptions>> {
    let mut profiles = Vec::new();
    for rule_id in rule_ids {
        for rule in config.rule_applications(rule_id) {
            let options: EmbeddedSqlRuleOptions = rule.try_rule_options()?;
            profiles.push(EmbeddedSqlOptions::configured(
                &options.import_specifier,
                &options.executor_names,
            ));
        }
    }
    profiles.sort();
    profiles.dedup();
    Ok(profiles)
}

#[inline(never)]
pub fn configured_embedded_sql_options_for_checks(
    config: &NoMistakesConfig,
) -> Result<Vec<EmbeddedSqlOptions>> {
    configured_embedded_sql_options(config, PREPARED_EMBEDDED_SQL_RULE_IDS)
}

#[inline(never)]
pub fn configured_schema_catalog_paths(
    config: &NoMistakesConfig,
    rule_ids: &[&str],
) -> Result<Vec<String>> {
    let mut paths = Vec::new();
    for rule_id in rule_ids {
        for rule in config.rule_applications(rule_id) {
            let options: CatalogRuleOptions = rule.try_rule_options()?;
            if !options.schema_catalog_path.is_empty() {
                paths.push(
                    super::catalog::normalize_catalog_path(&options.schema_catalog_path)?
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

/// Build one standalone request-scoped fact map for callers that do not
/// already own aggregate check facts. Every configured projection is derived
/// from the same parsed program for each file.
#[inline(never)]
pub(crate) fn prepare_embedded_sql_facts(
    root: &Path,
    files: &[PathBuf],
    sources: Arc<crate::codebase::ts_source::SourceStore>,
    profiles: Vec<EmbeddedSqlOptions>,
    postgres_schema_catalog_paths: Vec<String>,
) -> crate::codebase::check_facts::CheckFactMap {
    crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_and_sources(
        root,
        files.to_vec(),
        Vec::new(),
        crate::codebase::check_facts::CheckFactPlan {
            embedded_sql: !profiles.is_empty(),
            embedded_sql_options: profiles,
            postgres_schema_catalog_paths,
            ..Default::default()
        },
        None,
        sources,
    )
}

#[inline(never)]
pub(crate) fn load_schema_catalogs(
    root: &Path,
    sources: &crate::codebase::ts_source::SourceStore,
    paths: &[String],
) -> std::collections::BTreeMap<String, Result<Arc<super::SchemaCatalog>, Arc<str>>> {
    paths
        .iter()
        .map(|path| {
            let catalog = super::SchemaCatalog::load(root, path, sources)
                .map(Arc::new)
                .map_err(|error| Arc::<str>::from(error.to_string()));
            (path.clone(), catalog)
        })
        .collect()
}

#[cfg(test)]
mod tests;
