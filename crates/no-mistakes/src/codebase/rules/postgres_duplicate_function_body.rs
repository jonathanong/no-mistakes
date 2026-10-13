use super::RuleFinding;
use crate::codebase::postgres::{require_catalog_path, AllowEntry, AllowList};
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod normalize;
mod scan;
mod search_path;
mod token_class;

pub const RULE_ID: &str = "postgres-duplicate-function-body";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct Options {
    schema_catalog_path: String,
    min_cluster_size: Option<i64>,
    min_tokens: Option<i64>,
    normalize_identifiers: Option<bool>,
    normalize_raise: Option<bool>,
    keep_identifiers: Vec<String>,
    allow: Vec<AllowEntry>,
}

pub(crate) struct Compiled {
    schema_catalog_path: String,
    min_cluster_size: usize,
    min_tokens: usize,
    message: Option<String>,
    settings: normalize::Settings,
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
    Ok(Compiled {
        schema_catalog_path: opts.schema_catalog_path.clone(),
        min_cluster_size: minimum("minClusterSize", opts.min_cluster_size.unwrap_or(2), 2)?,
        min_tokens: minimum("minTokens", opts.min_tokens.unwrap_or(1), 1)?,
        message,
        settings: normalize::Settings {
            normalize_identifiers: opts.normalize_identifiers.unwrap_or(true),
            normalize_raise: opts.normalize_raise.unwrap_or(true),
            keep_identifiers: opts
                .keep_identifiers
                .iter()
                .map(|word| word.to_ascii_uppercase())
                .collect(),
        },
        allow: AllowList::compile(RULE_ID, opts.allow.clone())?,
    })
}

fn minimum(option: &str, value: i64, floor: i64) -> Result<usize> {
    if value < floor {
        bail!("{RULE_ID} option {option}: must be at least {floor}");
    }
    Ok(value as usize)
}

#[cfg(test)]
mod coverage;
#[cfg(test)]
mod tests;
