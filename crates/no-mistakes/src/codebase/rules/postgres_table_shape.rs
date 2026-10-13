use super::RuleFinding;
use crate::codebase::postgres::{AllowEntry, AllowList};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod check;
mod compile;
mod name;
mod scan;

pub const RULE_ID: &str = "postgres-table-shape";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct Options {
    schema_catalog_path: String,
    shapes: Vec<ShapeOptions>,
    banned_table_patterns: Vec<BannedOptions>,
    allow: Vec<AllowEntry>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct ShapeOptions {
    name: String,
    table_pattern: String,
    required_columns: Vec<ColumnOptions>,
    primary_key_types: Vec<String>,
    forbidden_columns: Vec<String>,
    required_triggers: Vec<TriggerOptions>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct ColumnOptions {
    name: Option<String>,
    name_pattern: Option<String>,
    #[serde(rename = "type")]
    data_type: Option<String>,
    nullable: Option<bool>,
    foreign_key: Option<bool>,
    on_delete: Option<String>,
    references: Option<Vec<String>>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct TriggerOptions {
    function: String,
    timing: Option<String>,
    events: Option<Vec<String>>,
    for_each_row: Option<bool>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct BannedOptions {
    pattern: String,
    message: String,
}

pub(crate) struct Compiled {
    schema_catalog_path: String,
    shapes: Vec<compile::Shape>,
    banned: Vec<compile::Banned>,
    allow: AllowList,
    message: Option<String>,
}

super::prepared_rule_entrypoints!(super::check_prepared_schema_rule, compile_applications);

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    _files: &[PathBuf],
    _sources: &crate::codebase::ts_source::SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let compiled = compile::compile(&rule.try_rule_options()?, rule.message.clone())?;
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
        .map(|rule| compile::compile(&rule.try_rule_options()?, rule.message.clone()))
        .collect()
}

#[cfg(test)]
mod tests;
