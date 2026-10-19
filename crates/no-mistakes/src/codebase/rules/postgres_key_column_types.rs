use crate::codebase::postgres::AllowEntry;
use crate::codebase::rules::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod compile;
mod scan;

pub const RULE_ID: &str = "postgres-key-column-types";

fn default_true() -> bool {
    true
}

#[derive(Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Options {
    pub(crate) schema_catalog_path: String,
    pub(crate) allowed_types: Vec<String>,
    pub(crate) allow_enum_types: bool,
    #[serde(default = "default_true")]
    pub(crate) check_primary_keys: bool,
    #[serde(default = "default_true")]
    pub(crate) check_foreign_keys: bool,
    pub(crate) allow: Vec<AllowEntry>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            schema_catalog_path: String::new(),
            allowed_types: Vec::new(),
            allow_enum_types: false,
            check_primary_keys: true,
            check_foreign_keys: true,
            allow: Vec::new(),
        }
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
    sources: &std::sync::Arc<crate::codebase::ts_source::SourceStore>,
) -> Result<Vec<RuleFinding>> {
    let paths = crate::codebase::postgres::configured_schema_catalog_paths(config, &[RULE_ID])?;
    let facts = crate::codebase::postgres::prepare_embedded_sql_facts(
        root,
        all_files,
        std::sync::Arc::clone(sources),
        Vec::new(),
        paths,
    );
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
        if filter.is_match(Path::new(&options.schema_catalog_path)) {
            let catalog = facts.postgres_schema_catalog(&options.schema_catalog_path)?;
            findings.extend(scan::scan(
                catalog,
                &compiled,
                &options.schema_catalog_path,
            )?);
        }
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

#[cfg(test)]
mod tests;
