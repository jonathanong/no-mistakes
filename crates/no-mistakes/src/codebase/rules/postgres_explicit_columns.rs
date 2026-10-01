use super::path_filter::GlobMatcher;
use super::RuleFinding;
use crate::codebase::postgres::{EmbeddedSqlOptions, PostgresSchemaOptions};
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod scan;

pub const RULE_ID: &str = "postgres-explicit-columns";

const DEFAULT_WHOLE_ROW: &[&str] = &[
    "row_to_json",
    "to_json",
    "to_jsonb",
    "json_agg",
    "jsonb_agg",
];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct Options {
    include: Vec<String>,
    exclude: Vec<String>,
    sql_include: Vec<String>,
    import_specifier: String,
    executor_names: Vec<String>,
    unanalyzable_sql: String,
    schema_catalog_path: String,
    max_columns: Option<i64>,
    relations: Vec<String>,
    allow_whole_row_functions: Option<Vec<String>>,
    check_returning: Option<bool>,
}

pub(crate) struct CompiledOptions {
    schema: PostgresSchemaOptions,
    embedded: EmbeddedSqlOptions,
    fail_unanalyzable: bool,
    schema_catalog_path: String,
    max_columns: u32,
    relations: Vec<String>,
    whole_row_functions: Vec<String>,
    check_returning: bool,
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(all_files);
    check_with_files_and_sources(root, config, all_files, &sources)
}

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    check_applications(root, config, all_files, sources, Some(facts))
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    check_applications(root, config, all_files, sources, None)
}

fn check_applications(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: Option<&crate::codebase::check_facts::CheckFactMap>,
) -> Result<Vec<RuleFinding>> {
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let opts: Options = rule.try_rule_options()?;
            let compiled = compile_options(&opts)?;
            let target_roots = super::target_roots(root, config, rule);
            let skip = super::skip_dir_set(config);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|path| {
                    super::file_allowed_by_roots_and_skip(root, &skip, path, &target_roots)
                })
                .cloned()
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            let include = GlobMatcher::new(&opts.include, &format!("{RULE_ID} include"))?;
            let exclude = GlobMatcher::new(&opts.exclude, &format!("{RULE_ID} exclude"))?;
            let files: Vec<PathBuf> = files
                .into_iter()
                .filter(|path| {
                    let rel = relative_slash_path(root, path);
                    (include.is_empty() || include.is_match(&rel))
                        && (exclude.is_empty() || !exclude.is_match(&rel))
                })
                .collect();
            scan::scan(root, &compiled, &files, sources, facts)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn compile_options(opts: &Options) -> Result<CompiledOptions> {
    if opts.schema_catalog_path.trim().is_empty() {
        bail!("{RULE_ID} option schemaCatalogPath: required");
    }
    let max_columns = match opts.max_columns {
        Some(value) if value < 0 => {
            bail!("{RULE_ID} option maxColumns: must be zero or greater")
        }
        Some(value) => value as u32,
        None => 12,
    };
    Ok(CompiledOptions {
        schema: PostgresSchemaOptions {
            sql_include: opts.sql_include.clone(),
        },
        embedded: EmbeddedSqlOptions::configured(&opts.import_specifier, &opts.executor_names),
        fail_unanalyzable: crate::codebase::postgres::fail_unanalyzable_sql(
            RULE_ID,
            &opts.unanalyzable_sql,
        )?,
        schema_catalog_path: opts.schema_catalog_path.clone(),
        max_columns,
        relations: unique_names(&opts.relations, "relations", "relation")?,
        whole_row_functions: match &opts.allow_whole_row_functions {
            Some(values) => unique_names(values, "allowWholeRowFunctions", "function")?,
            None => DEFAULT_WHOLE_ROW
                .iter()
                .map(|name| (*name).to_string())
                .collect(),
        },
        check_returning: opts.check_returning.unwrap_or(true),
    })
}

fn unique_names(values: &[String], option: &str, noun: &str) -> Result<Vec<String>> {
    let mut names = Vec::new();
    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            bail!("{RULE_ID} option {option}: empty {noun} name");
        }
        let key = trimmed.to_ascii_lowercase();
        if names.iter().any(|existing: &String| existing == &key) {
            bail!("{RULE_ID} option {option}: duplicate entry {key}");
        }
        names.push(key);
    }
    Ok(names)
}

#[cfg(test)]
mod options_tests;
#[cfg(test)]
mod tests;
