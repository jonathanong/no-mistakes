use super::RuleFinding;
use crate::codebase::postgres::{EmbeddedSqlOptions, PostgresSchemaOptions};
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
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
    executor_factory_names: Vec<String>,
    executor_type_names: Vec<String>,
    trusted_sql_tags: Vec<crate::codebase::postgres::TrustedSqlTag>,
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

super::prepared_rule_entrypoints!(super::check_prepared_sql_rule);

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    super::run_sql_rule_with_glob_filters(
        super::PreparedRuleContext::new(root, config, all_files, sources, facts),
        RULE_ID,
        compile_options,
        |opts: &Options| (&opts.include, &opts.exclude),
        |root, compiled, files, sources, facts| {
            scan::scan(root, compiled, files, sources, Some(facts))
        },
    )
}

fn compile_options(opts: &Options) -> Result<CompiledOptions> {
    if opts.schema_catalog_path.trim().is_empty() {
        bail!("{RULE_ID} option schemaCatalogPath: required");
    }
    let max_columns = match opts.max_columns {
        Some(value) if value < 0 => {
            bail!("{RULE_ID} option maxColumns: must be zero or greater")
        }
        Some(value) => u32::try_from(value).map_err(|_| {
            anyhow::anyhow!("{RULE_ID} option maxColumns: must be at most {}", u32::MAX)
        })?,
        None => 12,
    };
    Ok(CompiledOptions {
        schema: PostgresSchemaOptions {
            sql_include: opts.sql_include.clone(),
        },
        embedded: EmbeddedSqlOptions::configured(&opts.import_specifier, &opts.executor_names)
            .with_scoped_executors(&opts.executor_factory_names, &opts.executor_type_names)
            .with_trusted_sql_tags(&opts.trusted_sql_tags),
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

#[cfg(test)]
mod deferred_tests;

#[cfg(test)]
mod review_tests;
