use super::RuleFinding;
use crate::codebase::postgres::{AllowEntry, AllowList};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod check;
mod compile;
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
    _root: &Path,
    config: &NoMistakesConfig,
    _files: &[PathBuf],
    _sources: &crate::codebase::ts_source::SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for compiled in compile_applications(config)? {
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
        .map(|rule| compile::compile(&rule.try_rule_options()?))
        .collect()
}

#[cfg(test)]
mod tests;
