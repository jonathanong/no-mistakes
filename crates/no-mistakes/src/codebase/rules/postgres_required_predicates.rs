use super::path_filter::GlobMatcher;
use super::RuleFinding;
use crate::codebase::postgres::{EmbeddedSqlOptions, PostgresSchemaOptions};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod scan;

pub const RULE_ID: &str = "postgres-required-predicates";

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct RelationOption {
    pub(crate) table: String,
    pub(crate) require: Vec<String>,
    pub(crate) require_columns: Vec<String>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct PartitionExemption {
    pub(crate) table: String,
    pub(crate) reason: String,
}

use crate::codebase::postgres::AllowEntry;

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) include: Vec<String>,
    pub(crate) exclude: Vec<String>,
    pub(crate) sql_include: Vec<String>,
    pub(crate) import_specifier: String,
    pub(crate) executor_names: Vec<String>,
    pub(crate) executor_factory_names: Vec<String>,
    pub(crate) executor_type_names: Vec<String>,
    pub(crate) trusted_sql_tags: Vec<crate::codebase::postgres::TrustedSqlTag>,
    pub(crate) relations: Vec<RelationOption>,
    pub(crate) unanalyzable_sql: String,
    pub(crate) schema_catalog_path: String,
    pub(crate) partition_keys: String,
    pub(crate) partition_key_exemptions: Vec<PartitionExemption>,
    pub(crate) allow: Vec<AllowEntry>,
}

pub(crate) struct CompiledOptions {
    include: GlobMatcher,
    exclude: GlobMatcher,
    schema: PostgresSchemaOptions,
    embedded: EmbeddedSqlOptions,
    relations: Vec<RelationOption>,
    fail_unanalyzable: bool,
    schema_catalog_path: Option<String>,
    partition_keys: bool,
    partition_key_exemptions: Vec<PartitionExemption>,
    allow: Vec<AllowEntry>,
}

impl CompiledOptions {
    fn includes(&self, rel: &str) -> bool {
        (self.include.is_empty() || self.include.is_match(rel))
            && (self.exclude.is_empty() || !self.exclude.is_match(rel))
    }
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
        |compiled, rel| compiled.includes(rel),
        |root, compiled, files, sources, facts| {
            scan::scan(root, compiled, files, sources, Some(facts))
        },
    )
}

fn compile_options(opts: &Options) -> Result<CompiledOptions> {
    validate_options(opts)?;
    let include = GlobMatcher::new(&opts.include, &format!("{RULE_ID} include"))?;
    let exclude = GlobMatcher::new(&opts.exclude, &format!("{RULE_ID} exclude"))?;
    let partition_keys = opts.partition_keys == "require";
    Ok(CompiledOptions {
        include,
        exclude,
        schema: PostgresSchemaOptions {
            sql_include: if opts.sql_include.is_empty() {
                PostgresSchemaOptions::default().sql_include
            } else {
                opts.sql_include.clone()
            },
        },
        embedded: EmbeddedSqlOptions::configured(&opts.import_specifier, &opts.executor_names)
            .with_scoped_executors(&opts.executor_factory_names, &opts.executor_type_names)
            .with_trusted_sql_tags(&opts.trusted_sql_tags),
        relations: opts.relations.clone(),
        fail_unanalyzable: crate::codebase::postgres::fail_unanalyzable_sql(
            RULE_ID,
            &opts.unanalyzable_sql,
        )?,
        schema_catalog_path: (!opts.schema_catalog_path.trim().is_empty())
            .then(|| opts.schema_catalog_path.clone()),
        partition_keys,
        partition_key_exemptions: opts.partition_key_exemptions.clone(),
        allow: opts.allow.clone(),
    })
}

fn validate_options(opts: &Options) -> Result<()> {
    validate::check(opts)
}

mod validate;

#[cfg(test)]
mod columns_tests;
#[cfg(test)]
mod equality_tests;
#[cfg(test)]
mod options_tests;
#[cfg(test)]
mod scoped_tagged_tests;
#[cfg(test)]
mod tagged_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod trusted_tag_tests;

#[cfg(test)]
mod deferred_tests;
