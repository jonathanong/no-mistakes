use super::path_filter::GlobMatcher;
use super::RuleFinding;
use crate::codebase::postgres::{
    AllowEntry, AllowList, EmbeddedSqlOptions, PostgresSchemaOptions, SqlBoundKind,
};
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod arrays;
mod evaluate;
mod scan;

pub const RULE_ID: &str = "postgres-bounded-statements";

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
    unanalyzable_sql: String,
    schema_catalog_path: String,
    statements: Option<Vec<String>>,
    allow: Vec<AllowEntry>,
}

pub(crate) struct CompiledOptions {
    schema: PostgresSchemaOptions,
    embedded: EmbeddedSqlOptions,
    fail_unanalyzable: bool,
    schema_catalog_path: String,
    statements: Vec<SqlBoundKind>,
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
    sources: &std::sync::Arc<crate::codebase::ts_source::SourceStore>,
) -> Result<Vec<RuleFinding>> {
    let facts = crate::codebase::postgres::prepare_rule_sql_facts(
        root,
        all_files,
        std::sync::Arc::clone(sources),
        config,
        &[RULE_ID],
    )?;
    check_applications(root, config, all_files, sources, Some(&facts))
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
    crate::codebase::postgres::require_catalog_path(RULE_ID, &opts.schema_catalog_path)?;
    Ok(CompiledOptions {
        schema: PostgresSchemaOptions {
            sql_include: opts.sql_include.clone(),
        },
        embedded: EmbeddedSqlOptions::configured(&opts.import_specifier, &opts.executor_names)
            .with_scoped_executors(&opts.executor_factory_names, &opts.executor_type_names),
        fail_unanalyzable: crate::codebase::postgres::fail_unanalyzable_sql(
            RULE_ID,
            &opts.unanalyzable_sql,
        )?,
        schema_catalog_path: opts.schema_catalog_path.clone(),
        statements: statement_kinds(opts.statements.as_deref())?,
        allow: AllowList::compile(RULE_ID, opts.allow.clone())?,
    })
}

fn statement_kinds(configured: Option<&[String]>) -> Result<Vec<SqlBoundKind>> {
    let Some(configured) = configured else {
        return Ok(vec![
            SqlBoundKind::Select,
            SqlBoundKind::Update,
            SqlBoundKind::Delete,
        ]);
    };
    if configured.is_empty() {
        bail!("{RULE_ID} option statements: must name at least one of select, update, delete");
    }
    let mut kinds = Vec::new();
    for name in configured {
        let kind = match name.trim().to_ascii_lowercase().as_str() {
            "select" => SqlBoundKind::Select,
            "update" => SqlBoundKind::Update,
            "delete" => SqlBoundKind::Delete,
            _ => bail!("{RULE_ID} option statements: unknown statement {name}; use select, update or delete"),
        };
        if kinds.contains(&kind) {
            bail!("{RULE_ID} option statements: duplicate entry {name}");
        }
        kinds.push(kind);
    }
    Ok(kinds)
}

#[cfg(test)]
mod options_tests;
#[cfg(test)]
mod tests;

/// Unstable adapter for the prepared evaluator's Criterion harness.
#[cfg(feature = "test-instrumentation")]
pub(crate) fn benchmark_offenders(
    facts: &crate::codebase::postgres::SqlStatementFileFacts,
    catalog: &crate::codebase::postgres::SchemaCatalog,
) -> usize {
    facts
        .bounds
        .iter()
        .map(|fact| evaluate::offenders(fact, catalog).len())
        .sum()
}
