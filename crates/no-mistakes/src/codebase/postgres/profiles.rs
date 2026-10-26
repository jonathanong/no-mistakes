use super::{EmbeddedSqlOptions, TrustedSqlTag};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Filesystem rules whose configured executor projections must be declared at
/// the request boundary before TS/JS fact collection begins.
#[doc(hidden)]
pub const PREPARED_EMBEDDED_SQL_RULE_IDS: &[&str] = &[
    "postgres-idempotent-insert",
    "postgres-conflict-ordering",
    "postgres-lock-ordering",
    "postgres-required-predicates",
    "postgres-generated-column-predicates",
    "postgres-explicit-columns",
    "postgres-no-test-state-mutation",
    "postgres-bounded-statements",
    "postgres-no-offset",
    "postgres-no-transformed-uuid-predicates",
    "postgres-no-mutable-sql-registry",
    "postgres-no-unbounded-sql-identifier",
    "postgres-sql-shape-policy",
    "postgres-sql-statement-policy",
    "postgres-no-generated-column-writes",
    "postgres-require-query-annotation",
    "postgres-json-relation-ids",
];

/// Rules whose `schemaCatalogPath` is loaded for the request.
///
/// Schema-only rules append their ids here. This is not an alias of
/// [`PREPARED_EMBEDDED_SQL_RULE_IDS`].
#[doc(hidden)]
pub const SCHEMA_CATALOG_RULE_IDS: &[&str] = &[
    "postgres-column-requires-trigger",
    "postgres-conflict-ordering",
    "postgres-lock-ordering",
    "postgres-required-comments",
    "postgres-duplicate-function-body",
    "postgres-table-shape",
    "postgres-status-with-lifecycle-timestamps",
    "postgres-object-naming",
    "postgres-column-naming",
    "postgres-finite-text-columns",
    "postgres-array-columns",
    "postgres-key-column-types",
    "postgres-no-transformed-uuid-predicates",
    "postgres-required-predicates",
    "postgres-explicit-columns",
    "postgres-bounded-statements",
    "postgres-json-relation-ids",
];

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct EmbeddedSqlRuleOptions {
    import_specifier: Option<String>,
    executor_names: Vec<String>,
    executor_factory_names: Vec<String>,
    executor_type_names: Vec<String>,
    trusted_sql_tags: Vec<TrustedSqlTag>,
    sql_literal_helpers: Vec<String>,
    include: Vec<String>,
    test_globs: Vec<String>,
    helper_roots: Vec<String>,
    exclude: Vec<String>,
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct CatalogRuleOptions {
    schema_catalog_path: String,
}

/// Resolve every distinct embedded-SQL projection requested by the supplied
/// configured rules. Unknown rule-specific options are deliberately ignored;
/// their owning rule validates the complete option object.
pub(crate) fn configured_embedded_sql_options(
    config: &NoMistakesConfig,
    rule_ids: &[&str],
) -> Result<Vec<EmbeddedSqlOptions>> {
    Ok(configured_embedded_sql_options_with_annotation(config, rule_ids)?.0)
}

type AnnotatedSqlProfiles = (
    Vec<EmbeddedSqlOptions>,
    Vec<EmbeddedSqlOptions>,
    scans::EmbeddedSqlRuleScans,
);

pub fn configured_embedded_sql_options_for_checks(
    config: &NoMistakesConfig,
) -> Result<Vec<EmbeddedSqlOptions>> {
    configured_embedded_sql_options(config, PREPARED_EMBEDDED_SQL_RULE_IDS)
}

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

mod sql_include;
#[cfg(test)]
mod tests;
use sql_include::sql_patterns;

mod prepare;
pub(crate) mod scans;
pub(crate) use prepare::prepare_rule_sql_facts;
mod annotation_effort;
mod collect;
mod plan;
use collect::configured_embedded_sql_options_with_annotation;
pub use plan::{assign_prepared_embedded_sql_options, configure_prepared_postgres_plan};
