use super::RuleFinding;
use crate::codebase::postgres::{
    require_catalog_path, AllowEntry, AllowList, TriggerEvent, TriggerTiming,
};
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod scan;

pub const RULE_ID: &str = "postgres-column-requires-trigger";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct Options {
    schema_catalog_path: String,
    requirements: Vec<RequirementOptions>,
    allow: Vec<AllowEntry>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct RequirementOptions {
    column: String,
    function: String,
    timing: Option<String>,
    events: Option<Vec<String>>,
    for_each_row: Option<bool>,
    allow_column_list: Option<bool>,
}

pub(crate) struct Compiled {
    schema_catalog_path: String,
    requirements: Vec<Requirement>,
    allow: AllowList,
}

pub(crate) struct Requirement {
    column: String,
    function: String,
    timing: TriggerTiming,
    events: Vec<TriggerEvent>,
    for_each_row: bool,
    allow_column_list: bool,
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
        .map(|rule| compile_options(&rule.try_rule_options()?))
        .collect()
}

fn compile_options(opts: &Options) -> Result<Compiled> {
    require_catalog_path(RULE_ID, &opts.schema_catalog_path)?;
    let requirements = opts
        .requirements
        .iter()
        .map(compile_requirement)
        .collect::<Result<Vec<_>>>()?;
    Ok(Compiled {
        schema_catalog_path: opts.schema_catalog_path.clone(),
        requirements,
        allow: AllowList::compile(RULE_ID, opts.allow.clone())?,
    })
}

fn compile_requirement(requirement: &RequirementOptions) -> Result<Requirement> {
    if requirement.column.trim().is_empty() {
        bail!("{RULE_ID} option column: required");
    }
    if requirement.function.trim().is_empty() {
        bail!("{RULE_ID} option function: required");
    }
    Ok(Requirement {
        column: requirement.column.clone(),
        function: normalize_function_name(&requirement.function),
        timing: parse_timing(requirement.timing.as_deref().unwrap_or("before"))?,
        events: parse_events(requirement.events.as_deref())?,
        for_each_row: requirement.for_each_row.unwrap_or(true),
        allow_column_list: requirement.allow_column_list.unwrap_or(false),
    })
}

fn parse_timing(raw: &str) -> Result<TriggerTiming> {
    match raw {
        "before" => Ok(TriggerTiming::Before),
        "after" => Ok(TriggerTiming::After),
        "instead-of" => Ok(TriggerTiming::InsteadOf),
        _ => bail!("{RULE_ID} option timing: unknown value {raw}"),
    }
}

fn parse_events(events: Option<&[String]>) -> Result<Vec<TriggerEvent>> {
    let Some(events) = events else {
        return Ok(vec![TriggerEvent::Update]);
    };
    if events.is_empty() {
        bail!("{RULE_ID} option events: must not be empty");
    }
    events.iter().map(|event| parse_event(event)).collect()
}

/// Unquoted identifiers fold to lowercase. A quoted last segment keeps its case.
fn normalize_function_name(name: &str) -> String {
    let trimmed = name.trim();
    let last = trimmed.rsplit('.').next().unwrap_or(trimmed).trim();
    let Some(quoted) = last.strip_prefix('"').and_then(|rest| rest.strip_suffix('"')) else {
        return last.to_ascii_lowercase();
    };
    quoted.replace("\"\"", "\"")
}

fn parse_event(raw: &str) -> Result<TriggerEvent> {
    match raw {
        "insert" => Ok(TriggerEvent::Insert),
        "update" => Ok(TriggerEvent::Update),
        "delete" => Ok(TriggerEvent::Delete),
        "truncate" => Ok(TriggerEvent::Truncate),
        _ => bail!("{RULE_ID} option events: unknown event {raw}"),
    }
}

#[cfg(test)]
mod tests;
