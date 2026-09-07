use super::path_filter::GlobMatcher;
use super::RuleFinding;
use crate::codebase::postgres::{EmbeddedSqlOptions, PostgresSchemaOptions};
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

mod scan;

pub const RULE_ID: &str = "postgres-idempotent-insert";

fn default_true() -> bool {
    true
}

#[derive(Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) include: Vec<String>,
    pub(crate) exclude: Vec<String>,
    pub(crate) sql_include: Vec<String>,
    pub(crate) import_specifier: String,
    pub(crate) executor_names: Vec<String>,
    pub(crate) unanalyzable_sql: String,
    #[serde(default = "default_true")]
    pub(crate) scan_embedded: bool,
    #[serde(default = "default_true")]
    pub(crate) check_convergence: bool,
    #[serde(default = "default_true")]
    pub(crate) check_volatility: bool,
    #[serde(default = "default_true")]
    pub(crate) check_arbiter: bool,
    #[serde(default = "default_true")]
    pub(crate) check_triggers: bool,
    #[serde(default = "default_true")]
    pub(crate) check_generated: bool,
    pub(crate) replay_safe_trigger_functions: Vec<String>,
    pub(crate) trigger_written_columns: HashMap<String, Vec<String>>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            include: Vec::new(),
            exclude: Vec::new(),
            sql_include: Vec::new(),
            import_specifier: String::new(),
            executor_names: Vec::new(),
            unanalyzable_sql: String::new(),
            scan_embedded: true,
            check_convergence: true,
            check_volatility: true,
            check_arbiter: true,
            check_triggers: true,
            check_generated: true,
            replay_safe_trigger_functions: Vec::new(),
            trigger_written_columns: HashMap::new(),
        }
    }
}

pub(crate) struct CompiledOptions {
    include: GlobMatcher,
    exclude: GlobMatcher,
    schema: PostgresSchemaOptions,
    embedded: EmbeddedSqlOptions,
    fail_unanalyzable: bool,
    scan_embedded: bool,
    check_convergence: bool,
    check_volatility: bool,
    check_arbiter: bool,
    check_triggers: bool,
    check_generated: bool,
    replay_safe: Vec<String>,
    trigger_writes: Vec<(String, Vec<String>)>,
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

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.try_rule_options()?;
        let compiled = compile_options(&opts)?;
        let target_roots = super::target_roots(root, config, rule);
        let skip = super::skip_dir_set(config);
        let files: Vec<PathBuf> = all_files
            .iter()
            .filter(|path| super::file_allowed_by_roots_and_skip(root, &skip, path, &target_roots))
            .cloned()
            .collect();
        let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
        let files: Vec<PathBuf> = files
            .into_iter()
            .filter(|path| compiled.includes(&relative_slash_path(root, path)))
            .collect();
        findings.extend(scan::scan(root, &compiled, &files, sources)?);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn compile_options(opts: &Options) -> Result<CompiledOptions> {
    let include = GlobMatcher::new(&opts.include, &format!("{RULE_ID} include"))?;
    let exclude = GlobMatcher::new(&opts.exclude, &format!("{RULE_ID} exclude"))?;
    let defaults = EmbeddedSqlOptions::default();
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
        fail_unanalyzable: crate::codebase::postgres::fail_unanalyzable_sql(
            RULE_ID,
            &opts.unanalyzable_sql,
        )?,
        scan_embedded: opts.scan_embedded,
        check_convergence: opts.check_convergence,
        check_volatility: opts.check_volatility,
        check_arbiter: opts.check_arbiter,
        check_triggers: opts.check_triggers,
        check_generated: opts.check_generated,
        replay_safe: opts.replay_safe_trigger_functions.clone(),
        trigger_writes: opts
            .trigger_written_columns
            .iter()
            .map(|(name, columns)| (name.clone(), columns.clone()))
            .collect(),
    })
}

#[cfg(test)]
mod options_tests;
#[cfg(test)]
mod tests;
