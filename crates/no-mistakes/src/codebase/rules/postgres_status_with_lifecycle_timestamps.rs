use super::RuleFinding;
use crate::codebase::postgres::{require_catalog_path, AllowEntry, AllowList};
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod scan;

pub const RULE_ID: &str = "postgres-status-with-lifecycle-timestamps";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct Options {
    schema_catalog_path: String,
    status_columns: Vec<String>,
    lifecycle_verbs: Vec<String>,
    min_lifecycle_columns: Option<i64>,
    allow: Vec<AllowEntry>,
}

pub(crate) struct Compiled {
    schema_catalog_path: String,
    status_columns: Vec<String>,
    lifecycle_verbs: Vec<String>,
    min_lifecycle_columns: usize,
    allow: AllowList,
    message: Option<String>,
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
    sources: &Arc<crate::codebase::ts_source::SourceStore>,
) -> Result<Vec<RuleFinding>> {
    let _ = compile_applications(config)?;
    let catalog_paths =
        crate::codebase::postgres::configured_schema_catalog_paths(config, &[RULE_ID])?;
    let facts = crate::codebase::postgres::prepare_embedded_sql_facts(
        root,
        all_files,
        Arc::clone(sources),
        Vec::new(),
        catalog_paths,
    );
    check_with_files_sources_and_facts(root, config, all_files, sources, &facts)
}

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    _files: &[PathBuf],
    _sources: &crate::codebase::ts_source::SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let compiled = compile_options(&rule.try_rule_options()?, rule.message.clone())?;
        let filter = super::path_filter::RulePathFilter::new(root, config, rule)?;
        if !filter.is_match(Path::new(&compiled.schema_catalog_path)) {
            continue;
        }
        let catalog = facts.postgres_schema_catalog(&compiled.schema_catalog_path)?;
        findings.extend(scan::scan(compiled, catalog));
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn compile_applications(config: &NoMistakesConfig) -> Result<Vec<Compiled>> {
    config
        .rule_applications(RULE_ID)
        .into_iter()
        .map(|rule| compile_options(&rule.try_rule_options()?, rule.message.clone()))
        .collect()
}

fn compile_options(opts: &Options, message: Option<String>) -> Result<Compiled> {
    require_catalog_path(RULE_ID, &opts.schema_catalog_path)?;
    let min_lifecycle_columns =
        usize::try_from(opts.min_lifecycle_columns.unwrap_or(2)).unwrap_or(0);
    if min_lifecycle_columns < 1 {
        bail!("{RULE_ID} option minLifecycleColumns: must be at least 1");
    }
    Ok(Compiled {
        schema_catalog_path: opts.schema_catalog_path.clone(),
        status_columns: compile_words("statusColumns", &opts.status_columns, true)?,
        lifecycle_verbs: compile_words("lifecycleVerbs", &opts.lifecycle_verbs, false)?,
        min_lifecycle_columns,
        allow: AllowList::compile(RULE_ID, opts.allow.clone())?,
        message,
    })
}

fn compile_words(option: &str, words: &[String], required: bool) -> Result<Vec<String>> {
    if required && words.is_empty() {
        bail!("{RULE_ID} option {option}: must not be empty");
    }
    let mut compiled = Vec::new();
    for word in words {
        let word = word.trim();
        if word.is_empty() {
            bail!("{RULE_ID} option {option}: empty string");
        }
        if compiled.iter().any(|existing: &String| existing == word) {
            bail!("{RULE_ID} option {option}: duplicate entry {word}");
        }
        compiled.push(word.to_string());
    }
    Ok(compiled)
}

#[cfg(test)]
mod tests;
