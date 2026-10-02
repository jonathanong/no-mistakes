use crate::codebase::postgres::AllowEntry;
use crate::codebase::rules::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod compile;
mod parse;
mod pin;
mod scan;
mod text;

pub const RULE_ID: &str = "postgres-finite-text-columns";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) schema_catalog_path: String,
    #[serde(default = "default_column_types")]
    pub(crate) column_types: Vec<String>,
    pub(crate) name_patterns: Vec<String>,
    pub(crate) skip_generated_columns: bool,
    pub(crate) ignore_table_patterns: Vec<String>,
    pub(crate) allow: Vec<AllowEntry>,
}

fn default_column_types() -> Vec<String> {
    vec!["text".to_string(), "character varying".to_string()]
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
        let mut rule_findings = scan::scan(catalog, &compiled, &options.schema_catalog_path);
        if let Some(message) = compiled
            .message
            .as_deref()
            .filter(|message| !message.trim().is_empty())
        {
            for finding in &mut rule_findings {
                finding.message = format!(
                    "{}: {}: {message}",
                    finding.file,
                    finding.target.as_deref().unwrap_or_default()
                );
            }
        }
        findings.extend(rule_findings);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

#[cfg(test)]
mod tests;
