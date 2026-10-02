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
mod validate;

pub const RULE_ID: &str = "postgres-generated-column-predicates";

pub(crate) use check_with_files as check;

use crate::codebase::ts_source::TS_JS_EXTENSIONS;

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct ExtraGeneratedColumn {
    pub(crate) table: String,
    pub(crate) column: String,
    pub(crate) source_column: String,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct Options {
    include: Vec<String>,
    exclude: Vec<String>,
    sql_include: Vec<String>,
    import_specifier: String,
    executor_names: Vec<String>,
    unanalyzable_sql: String,
    functions: Option<Vec<String>>,
    require_argument_is_primary_key: Option<bool>,
    clauses: Option<Vec<String>>,
    extra_generated_columns: Vec<ExtraGeneratedColumn>,
}

pub(crate) struct CompiledOptions {
    schema: PostgresSchemaOptions,
    embedded: EmbeddedSqlOptions,
    fail_unanalyzable: bool,
    functions: Vec<String>,
    require_primary_key: bool,
    where_clause: bool,
    join_clause: bool,
    order_clause: bool,
    extras: Vec<ExtraGeneratedColumn>,
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
    check_applications(root, config, all_files, sources, facts)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &std::sync::Arc<crate::codebase::ts_source::SourceStore>,
) -> Result<Vec<RuleFinding>> {
    let facts = crate::codebase::postgres::prepare_rule_sql_facts(
        root,
        all_files,
        std::sync::Arc::clone(sources),
        config,
        &[RULE_ID],
    )?;
    check_applications(root, config, all_files, sources, &facts)
}

fn check_applications(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
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
            let query_files: Vec<PathBuf> = files
                .iter()
                .filter(|path| allowed(root, path, &include, &exclude))
                .cloned()
                .collect();
            let schema_files =
                crate::codebase::postgres::postgres_sql_paths(root, &files, &compiled.schema)?;
            let query_set = crate::codebase::check_facts::PathMembership::new(&query_files);
            let schema_set = crate::codebase::check_facts::PathMembership::new(&schema_files);
            let files: Vec<_> = files
                .into_iter()
                .filter(|path| query_set.contains(path) || schema_set.contains(path))
                .collect();
            scan::scan(root, &compiled, &files, &query_files, sources, facts)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn allowed(root: &Path, path: &Path, include: &GlobMatcher, exclude: &GlobMatcher) -> bool {
    let rel = relative_slash_path(root, path);
    let included = if include.is_empty() {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext == "sql" || TS_JS_EXTENSIONS.contains(&ext))
    } else {
        include.is_match(&rel)
    };
    included && (exclude.is_empty() || !exclude.is_match(&rel))
}

fn compile_options(opts: &Options) -> Result<CompiledOptions> {
    let functions = match &opts.functions {
        None => vec!["uuid_extract_timestamp".to_string()],
        Some(values) if values.is_empty() => {
            bail!("{RULE_ID} option functions: must not be empty")
        }
        Some(values) => validate::unique_names(values, "functions", "function")?,
    };
    let (where_clause, join_clause, order_clause) = validate::clauses(opts.clauses.as_deref())?;
    Ok(CompiledOptions {
        schema: PostgresSchemaOptions {
            sql_include: if opts.sql_include.is_empty() {
                PostgresSchemaOptions::default().sql_include
            } else {
                opts.sql_include.clone()
            },
        },
        embedded: EmbeddedSqlOptions::configured(&opts.import_specifier, &opts.executor_names),
        fail_unanalyzable: crate::codebase::postgres::fail_unanalyzable_sql(
            RULE_ID,
            &opts.unanalyzable_sql,
        )?,
        functions,
        require_primary_key: opts.require_argument_is_primary_key.unwrap_or(true),
        where_clause,
        join_clause,
        order_clause,
        extras: validate::extras(&opts.extra_generated_columns)?,
    })
}

#[cfg(test)]
mod options_tests;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod followup_tests;

#[cfg(test)]
mod history_tests;
