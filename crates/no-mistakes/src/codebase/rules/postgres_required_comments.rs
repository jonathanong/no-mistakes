use super::RuleFinding;
use crate::codebase::postgres::{require_catalog_path, AllowEntry, AllowList};
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
use regex::Regex;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;

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
        let compiled = compile_options(&rule.try_rule_options()?)?;
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
