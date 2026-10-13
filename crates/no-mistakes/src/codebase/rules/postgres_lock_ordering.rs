use super::RuleFinding;
use crate::codebase::postgres::{fail_unanalyzable_sql, EmbeddedSqlOptions};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod catalog_check;
pub(crate) mod directive;
mod scan;

pub const RULE_ID: &str = "postgres-lock-ordering";

use directive::DEFAULT_SAFE_DIRECTIVE;
use scan::scan_with_sources;

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) include: Vec<String>,
    pub(crate) exclude: Vec<String>,
    pub(crate) import_specifier: String,
    pub(crate) executor_names: Vec<String>,
    pub(crate) executor_factory_names: Vec<String>,
    pub(crate) executor_type_names: Vec<String>,
    pub(crate) trusted_sql_tags: Vec<crate::codebase::postgres::TrustedSqlTag>,
    pub(crate) safe_directive: String,
    pub(crate) schema_catalog_path: String,
    pub(crate) unanalyzable_sql: String,
}

struct CompiledOptions {
    paths: super::RuleGlobFilter,
    embedded: EmbeddedSqlOptions,
    safe_directive: String,
    schema_catalog_path: Option<String>,
    fail_unanalyzable: bool,
}

super::prepared_rule_entrypoints!(super::check_prepared_sql_rule);

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    super::run_included_postgres_rule(
        super::PreparedRuleContext::new(root, config, all_files, sources, facts),
        RULE_ID,
        |rule| compile_options(&rule.try_rule_options()?),
        |compiled, rel| compiled.paths.includes(rel),
        scan_with_sources,
    )
}

fn compile_options(opts: &Options) -> Result<CompiledOptions> {
    Ok(CompiledOptions {
        paths: super::RuleGlobFilter::new(RULE_ID, &opts.include, &opts.exclude)?,
        embedded: EmbeddedSqlOptions::configured(&opts.import_specifier, &opts.executor_names)
            .with_scoped_executors(&opts.executor_factory_names, &opts.executor_type_names)
            .with_trusted_sql_tags(&opts.trusted_sql_tags),
        safe_directive: if opts.safe_directive.is_empty() {
            DEFAULT_SAFE_DIRECTIVE.to_string()
        } else {
            opts.safe_directive.clone()
        },
        schema_catalog_path: (!opts.schema_catalog_path.is_empty())
            .then(|| opts.schema_catalog_path.clone()),
        fail_unanalyzable: fail_unanalyzable_sql(RULE_ID, &opts.unanalyzable_sql)?,
    })
}

#[cfg(test)]
mod tests;
