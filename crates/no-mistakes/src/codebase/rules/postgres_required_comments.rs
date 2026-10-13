use super::RuleFinding;
use crate::codebase::postgres::{require_catalog_path, AllowEntry, AllowList};
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
use regex::Regex;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod scan;

pub const RULE_ID: &str = "postgres-required-comments";

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ObjectKind {
    Table,
    View,
    MaterializedView,
    Column,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct Options {
    schema_catalog_path: String,
    objects: Option<Vec<String>>,
    column_name_patterns: Vec<String>,
    exempt_column_name_patterns: Vec<String>,
    min_length: Option<i64>,
    allow: Vec<AllowEntry>,
}

pub(crate) struct Compiled {
    schema_catalog_path: String,
    objects: Vec<ObjectKind>,
    column_name_patterns: Vec<Regex>,
    exempt_column_name_patterns: Vec<Regex>,
    min_length: u64,
    allow: AllowList,
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
    let min_length = u64::try_from(opts.min_length.unwrap_or(1)).unwrap_or(0);
    if min_length < 1 {
        bail!("{RULE_ID} option minLength: must be at least 1");
    }
    Ok(Compiled {
        schema_catalog_path: opts.schema_catalog_path.clone(),
        objects: compile_objects(opts.objects.as_deref())?,
        column_name_patterns: compile_patterns("columnNamePatterns", &opts.column_name_patterns)?,
        exempt_column_name_patterns: compile_patterns(
            "exemptColumnNamePatterns",
            &opts.exempt_column_name_patterns,
        )?,
        min_length,
        allow: AllowList::compile(RULE_ID, opts.allow.clone())?,
    })
}

fn compile_objects(objects: Option<&[String]>) -> Result<Vec<ObjectKind>> {
    let Some(objects) = objects else {
        return Ok(vec![ObjectKind::Table]);
    };
    if objects.is_empty() {
        bail!("{RULE_ID} option objects: must not be empty");
    }
    let mut kinds = Vec::new();
    for raw in objects {
        let kind = parse_object(raw)?;
        if kinds.contains(&kind) {
            bail!("{RULE_ID} option objects: duplicate entry {raw}");
        }
        kinds.push(kind);
    }
    Ok(kinds)
}

fn parse_object(raw: &str) -> Result<ObjectKind> {
    match raw {
        "table" => Ok(ObjectKind::Table),
        "view" => Ok(ObjectKind::View),
        "materialized-view" => Ok(ObjectKind::MaterializedView),
        "column" => Ok(ObjectKind::Column),
        _ => bail!("{RULE_ID} option objects: unknown value {raw}"),
    }
}

fn compile_patterns(option: &str, patterns: &[String]) -> Result<Vec<Regex>> {
    patterns
        .iter()
        .map(|pattern| {
            Regex::new(pattern).map_err(|error| {
                anyhow::anyhow!("{RULE_ID} option {option}: invalid regex {pattern}: {error}")
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;
