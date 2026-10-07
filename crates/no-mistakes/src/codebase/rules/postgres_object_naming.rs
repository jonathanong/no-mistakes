use crate::codebase::postgres::AllowEntry;
use crate::codebase::rules::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

mod compile;
mod compile_spelling;
mod expand;
mod name_flags;
mod pattern;
mod pattern_bounds;
mod pattern_flags;
mod pattern_walk;
mod policy;
mod replacement;
mod scan;
mod scan_name;

pub const RULE_ID: &str = "postgres-object-naming";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) schema_catalog_path: String,
    pub(crate) patterns: BTreeMap<String, String>,
    pub(crate) check_constraint_backed_indexes: bool,
    pub(crate) table_min_words: Option<i64>,
    pub(crate) abbreviations: Abbreviations,
    pub(crate) plural: PluralOptions,
    pub(crate) denied_tokens: Vec<DeniedToken>,
    pub(crate) spelling: BTreeMap<String, String>,
    pub(crate) double_underscore: Option<DoubleUnderscore>,
    pub(crate) allow: Vec<AllowEntry>,
}

#[derive(Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Abbreviations {
    pub(crate) enabled: bool,
    #[serde(default = "default_min_letters")]
    pub(crate) min_letters: i64,
}

fn default_min_letters() -> i64 {
    3
}

impl Default for Abbreviations {
    fn default() -> Self {
        Self {
            enabled: false,
            min_letters: default_min_letters(),
        }
    }
}

#[derive(Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct PluralOptions {
    pub(crate) enabled: bool,
    #[serde(default = "default_plural_objects")]
    pub(crate) objects: Vec<String>,
    pub(crate) irregular_plurals: BTreeMap<String, String>,
    pub(crate) uncountable: Vec<String>,
    pub(crate) non_plural_tokens: Vec<String>,
    pub(crate) ignore_patterns: Vec<String>,
}

fn default_plural_objects() -> Vec<String> {
    vec!["table".to_string()]
}

impl Default for PluralOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            objects: default_plural_objects(),
            irregular_plurals: BTreeMap::new(),
            uncountable: Vec::new(),
            non_plural_tokens: Vec::new(),
            ignore_patterns: Vec::new(),
        }
    }
}

#[derive(Deserialize, Clone, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct DeniedToken {
    pub(crate) token: String,
    pub(crate) replacement: String,
}

#[derive(Deserialize, Clone, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct DoubleUnderscore {
    pub(crate) allow_pattern: Option<String>,
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
    sources: &std::sync::Arc<crate::codebase::ts_source::SourceStore>,
) -> Result<Vec<RuleFinding>> {
    let catalog_paths =
        crate::codebase::postgres::configured_schema_catalog_paths(config, &[RULE_ID])?;
    let facts = crate::codebase::postgres::prepare_embedded_sql_facts(
        root,
        all_files,
        std::sync::Arc::clone(sources),
        Vec::new(),
        catalog_paths,
    );
    let _ = compile_applications(config)?;
    check_with_files_sources_and_facts(root, config, all_files, sources, &facts)
}

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    _all_files: &[PathBuf],
    _sources: &crate::codebase::ts_source::SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let options: Options = rule.try_rule_options()?;
        let compiled = compile::compile(&options, rule.message.clone())?;
        let filter = super::path_filter::RulePathFilter::new(root, config, rule)?;
        if !filter.is_match(Path::new(&options.schema_catalog_path)) {
            continue;
        }
        let catalog = facts.postgres_schema_catalog(&options.schema_catalog_path)?;
        findings.extend(scan::scan(catalog, &compiled, &options.schema_catalog_path));
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn compile_applications(config: &NoMistakesConfig) -> Result<Vec<compile::Compiled>> {
    config
        .rule_applications(RULE_ID)
        .into_iter()
        .map(|rule| compile::compile(&rule.try_rule_options()?, rule.message.clone()))
        .collect()
}

#[cfg(test)]
mod tests;
