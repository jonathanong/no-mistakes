use super::path_filter::GlobMatcher;
use super::RuleFinding;
use crate::codebase::postgres::{EmbeddedSqlOptions, PostgresSchemaOptions};
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
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

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct AllowEntry {
    pub(crate) object: String,
    pub(crate) reason: String,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) include: Vec<String>,
    pub(crate) exclude: Vec<String>,
    pub(crate) sql_include: Vec<String>,
    pub(crate) import_specifier: String,
    pub(crate) executor_names: Vec<String>,
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

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(all_files);
    check_with_files_and_sources(root, config, all_files, &sources)
}

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    check_applications(root, config, all_files, sources, Some(facts))
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    check_applications(root, config, all_files, sources, None)
}

fn check_applications(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: Option<&crate::codebase::check_facts::CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let opts: Options = rule.try_rule_options()?;
            let compiled = compile_options(&opts)?;
            let target_roots = super::target_roots(root, config, rule);
            let skip = super::skip_dir_set(config);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|path| {
                    super::file_allowed_by_roots_and_skip(root, &skip, path, &target_roots)
                })
                .cloned()
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            let files: Vec<PathBuf> = files
                .into_iter()
                .filter(|path| compiled.includes(&relative_slash_path(root, path)))
                .collect();
            scan::scan(root, &compiled, &files, sources, facts)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn compile_options(opts: &Options) -> Result<CompiledOptions> {
    validate_options(opts)?;
    let include = GlobMatcher::new(&opts.include, &format!("{RULE_ID} include"))?;
    let exclude = GlobMatcher::new(&opts.exclude, &format!("{RULE_ID} exclude"))?;
    let defaults = EmbeddedSqlOptions::default();
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
        embedded: EmbeddedSqlOptions {
            import_specifier: if opts.import_specifier.is_empty() {
                defaults.import_specifier
            } else {
                opts.import_specifier.clone()
            },
            executor_names: if opts.executor_names.is_empty() {
                defaults.executor_names
            } else {
                opts.executor_names.clone()
            },
        },
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
mod options_tests;
#[cfg(test)]
mod tests;
