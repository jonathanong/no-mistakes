use super::RuleFinding;
use crate::codebase::postgres::{
    require_catalog_path, AllowEntry, AllowList, TriggerEvent, TriggerTiming,
};
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod name;
mod scan;

use name::normalize_function_name;

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

super::prepared_rule_entrypoints!(super::check_prepared_schema_rule, compile_applications);

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    _files: &[PathBuf],
    _sources: &crate::codebase::ts_source::SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    super::run_catalog_rule(
        root,
        config,
        facts,
        RULE_ID,
        |rule| {
            let compiled = compile_options(&rule.try_rule_options()?)?;
            Ok((compiled.schema_catalog_path.clone(), compiled))
        },
        |compiled, catalog, _| scan::scan(compiled, catalog),
    )
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
    let events = parse_events(requirement.events.as_deref())?;
    let for_each_row = requirement.for_each_row.unwrap_or(true);
    if for_each_row && events.contains(&TriggerEvent::Truncate) {
        bail!("{RULE_ID} option forEachRow: truncate triggers are FOR EACH STATEMENT");
    }
    Ok(Requirement {
        column: requirement.column.clone(),
        function: normalize_function_name(&requirement.function),
        timing: parse_timing(requirement.timing.as_deref().unwrap_or("before"))?,
        events,
        for_each_row,
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
