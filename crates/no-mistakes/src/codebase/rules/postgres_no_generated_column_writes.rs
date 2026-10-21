use super::path_filter::GlobMatcher;
use super::RuleFinding;
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::{fail_unanalyzable_sql, EmbeddedSqlOptions, PostgresSchemaOptions};
use crate::codebase::ts_source::{discover_files, relative_slash_path};
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
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
    #[serde(default)]
    pub(crate) sql_include_locked: bool,
    pub(crate) include: Vec<String>,
    pub(crate) import_specifier: Option<String>,
    pub(crate) executor_names: Vec<String>,
    pub(crate) executor_factory_names: Vec<String>,
    pub(crate) executor_type_names: Vec<String>,
    pub(crate) trusted_sql_tags: Vec<crate::codebase::postgres::TrustedSqlTag>,
    pub(crate) extra_generated_columns: Vec<ExtraGeneratedColumn>,
    pub(crate) trigger_maintained_columns: Vec<String>,
    pub(crate) unanalyzable_sql: String,
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
    trigger_maintained_columns: Vec<String>,
    fail_unanalyzable: bool,
    advise_unanalyzable: bool,
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

super::prepared_rule_entrypoints!(super::check_prepared_sql_rule);

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let all: Result<Vec<super::PostgresFindings>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .enumerate()
        .map(|(index, rule)| -> Result<super::PostgresFindings> {
            let opts: Options = rule.try_rule_options()?;
            let compiled = compile_options(&opts)?;
            let files = super::selected_rule_files(root, config, rule, all_files)?;
            let view = facts.for_sql_application(index, rule.name.as_deref());
            scan::scan(
                root,
                &compiled,
                &files,
                sources,
                view.as_ref()
                    .map_or(facts as &dyn SqlFactContext, |view| view),
            )
        })
        .collect();
    let findings: super::PostgresFindings = all?.into_iter().flatten().collect();
    Ok(findings.finish())
}

fn compile_options(opts: &Options) -> Result<CompiledOptions> {
    let include = GlobMatcher::new(&opts.include, &format!("{RULE_ID} include"))?;
    Ok(CompiledOptions {
        include,
        schema: PostgresSchemaOptions {
            sql_include: crate::codebase::postgres::effective_sql_include(
                &opts.sql_include,
                opts.sql_include_locked,
            ),
        },
        embedded: embedded_options(opts),
        extra_generated_columns: opts.extra_generated_columns.clone(),
        trigger_maintained_columns: trigger_columns(&opts.trigger_maintained_columns)?,
        fail_unanalyzable: fail_unanalyzable_sql(RULE_ID, &opts.unanalyzable_sql)?,
        advise_unanalyzable: opts.unanalyzable_sql.trim().eq_ignore_ascii_case("advise"),
    })
}

fn trigger_columns(values: &[String]) -> Result<Vec<String>> {
    let mut names = Vec::new();
    let mut seen = Vec::new();
    for value in values {
        let name = value.trim();
        if name.is_empty() {
            bail!("{RULE_ID} option triggerMaintainedColumns: empty column name");
        }
        let key = name.to_ascii_lowercase();
        if seen.contains(&key) {
            bail!("{RULE_ID} option triggerMaintainedColumns: duplicate entry {key}");
        }
        seen.push(key);
        names.push(name.to_string());
    }
    Ok(names)
}

fn embedded_options(opts: &Options) -> EmbeddedSqlOptions {
    EmbeddedSqlOptions::configured(
        opts.import_specifier.as_deref().unwrap_or_default(),
        &opts.executor_names,
    )
    .with_scoped_executors(&opts.executor_factory_names, &opts.executor_type_names)
    .with_trusted_sql_tags(&opts.trusted_sql_tags)
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

fn trigger_finding(file: &str, line: usize, table: &str, column: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message: format!(
            "{file}:{line}: do not write trigger-maintained column `{table}.{column}`; \
it is listed in triggerMaintainedColumns, so the database sets it — remove it from the INSERT/UPDATE"
        ),
        import: Some(format!("{table}.{column}")),
        target: Some(column.to_string()),
    }
}

fn unanalyzable_finding(file: &str, line: usize) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message: format!(
            "{file}:{line}: executed SQL is not statically recoverable, so a write to a generated or trigger-maintained column cannot be ruled out; pass a SQL literal or trusted tagged template, or set unanalyzableSql: ignore"
        ),
        import: None,
        target: Some("unanalyzable".to_string()),
    }
}

#[cfg(test)]
mod tests;
