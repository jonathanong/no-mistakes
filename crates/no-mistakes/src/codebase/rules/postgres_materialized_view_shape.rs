use super::RuleFinding;
use crate::codebase::postgres::{require_catalog_path, AllowEntry, AllowList};
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

mod scan;
pub const RULE_ID: &str = "postgres-materialized-view-shape";

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct BaseTable {
    table: String,
    reason: String,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct Options {
    schema_catalog_path: String,
    min_additional_indexes: Option<u64>,
    window_functions: Vec<String>,
    allow_base_tables: Vec<BaseTable>,
    allow: Vec<AllowEntry>,
}

struct Compiled {
    path: String,
    additional: u64,
    functions: Vec<String>,
    base_tables: Vec<BaseTable>,
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
    super::run_rule_applications(config, RULE_ID, |rule| {
        let compiled = compile(&rule.try_rule_options()?)?;
        let Some(catalog) =
            super::selected_schema_catalog(root, config, rule, facts, &compiled.path)?
        else {
            return Ok(Vec::new());
        };
        let mut findings = scan::scan(compiled, catalog);
        if let Some(message) = &rule.message {
            for finding in &mut findings {
                finding.message = format!("{message} {}", finding.message);
            }
        }
        Ok(findings)
    })
}

fn compile_applications(config: &NoMistakesConfig) -> Result<Vec<Compiled>> {
    config
        .rule_applications(RULE_ID)
        .into_iter()
        .map(|rule| compile(&rule.try_rule_options()?))
        .collect()
}

fn compile(options: &Options) -> Result<Compiled> {
    require_catalog_path(RULE_ID, &options.schema_catalog_path)?;
    let mut seen = BTreeSet::new();
    for entry in &options.allow_base_tables {
        if entry.table.trim().is_empty() || entry.reason.trim().is_empty() {
            bail!("{RULE_ID} option allowBaseTables: each table needs a nonempty name and reason");
        }
        if !seen.insert(entry.table.clone()) {
            bail!(
                "{RULE_ID} option allowBaseTables: duplicate table {}",
                entry.table
            );
        }
    }
    if options
        .window_functions
        .iter()
        .any(|name| name.trim().is_empty())
    {
        bail!("{RULE_ID} option windowFunctions: names must not be empty");
    }
    let functions = options.window_functions.iter().map(|name| {
        crate::codebase::postgres::idents::parse_relation_identity(name).ok_or_else(|| {
            anyhow::anyhow!("{RULE_ID} option windowFunctions: expected a PostgreSQL function name, found {name}")
        })
    }).collect::<Result<Vec<_>>>()?;
    Ok(Compiled {
        path: options.schema_catalog_path.clone(),
        additional: options.min_additional_indexes.unwrap_or(1),
        functions,
        base_tables: options.allow_base_tables.clone(),
        allow: AllowList::compile(RULE_ID, options.allow.clone())?,
    })
}

#[cfg(test)]
mod tests;
