use super::path_filter::GlobMatcher;
use super::RuleFinding;
use crate::codebase::postgres::{EmbeddedSqlOptions, PostgresSchemaOptions};
use crate::codebase::ts_source::{discover_files, relative_slash_path};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod catalog;
mod scan;

pub const RULE_ID: &str = "postgres-no-generated-column-writes";

const DEFAULT_DML_EXTENSIONS: &[&str] = &["ts", "mts", "tsx", "js", "sql"];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) sql_include: Vec<String>,
    pub(crate) include: Vec<String>,
    pub(crate) import_specifier: Option<String>,
    pub(crate) executor_names: Vec<String>,
    pub(crate) extra_generated_columns: Vec<ExtraGeneratedColumn>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct ExtraGeneratedColumn {
    pub(crate) table: String,
    pub(crate) column: String,
}

struct CompiledOptions {
    include: GlobMatcher,
    schema: PostgresSchemaOptions,
    embedded: EmbeddedSqlOptions,
    extra_generated_columns: Vec<ExtraGeneratedColumn>,
}

impl CompiledOptions {
    fn includes_dml(&self, root: &Path, path: &Path) -> bool {
        let rel = relative_slash_path(root, path);
        if self.include.is_empty() {
            is_default_dml_path(path)
        } else {
            self.include.is_match(&rel)
        }
    }
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let files = discover_files(root, &config.filesystem.skip_directories);
    check_with_files(root, config, &files)
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
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let opts: Options = rule.rule_options()?;
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
            scan::scan_with_sources(root, &compiled, &files, sources)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn compile_options(opts: &Options) -> Result<CompiledOptions> {
    let include = GlobMatcher::new(&opts.include, &format!("{RULE_ID} include"))?;
    Ok(CompiledOptions {
        include,
        schema: PostgresSchemaOptions {
            sql_include: if opts.sql_include.is_empty() {
                PostgresSchemaOptions::default().sql_include
            } else {
                opts.sql_include.clone()
            },
        },
        embedded: embedded_options(opts),
        extra_generated_columns: opts.extra_generated_columns.clone(),
    })
}

fn embedded_options(opts: &Options) -> EmbeddedSqlOptions {
    let mut embedded = EmbeddedSqlOptions::default();
    if let Some(specifier) = opts.import_specifier.as_deref() {
        if !specifier.is_empty() {
            embedded.import_specifier = specifier.to_string();
        }
    }
    if !opts.executor_names.is_empty() {
        embedded.executor_names = opts.executor_names.clone();
    }
    embedded
}

fn is_default_dml_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| DEFAULT_DML_EXTENSIONS.contains(&extension))
}

fn finding(file: &str, line: usize, table: &str, column: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message: format!(
            "{file}:{line}: do not write generated column `{table}.{column}`; \
PostgreSQL computes GENERATED ALWAYS columns — omit it from INSERT/UPDATE and write the source column instead"
        ),
        import: Some(format!("{table}.{column}")),
        target: Some(column.to_string()),
    }
}

#[cfg(test)]
mod tests;
