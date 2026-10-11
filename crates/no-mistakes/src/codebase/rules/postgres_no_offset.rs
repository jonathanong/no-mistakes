use super::path_filter::GlobMatcher;
use super::RuleFinding;
use crate::codebase::postgres::{fail_unanalyzable_sql, EmbeddedSqlOptions};
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod scan;

pub const RULE_ID: &str = "postgres-no-offset";

use scan::scan;

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) include: Vec<String>,
    pub(crate) exclude: Vec<String>,
    pub(crate) sql_include: Vec<String>,
    pub(crate) import_specifier: String,
    pub(crate) executor_names: Vec<String>,
    pub(crate) executor_factory_names: Vec<String>,
    pub(crate) executor_type_names: Vec<String>,
    pub(crate) trusted_sql_tags: Vec<crate::codebase::postgres::TrustedSqlTag>,
    pub(crate) unanalyzable_sql: String,
}

pub(crate) struct CompiledOptions {
    include: GlobMatcher,
    exclude: GlobMatcher,
    pub(crate) schema: crate::codebase::postgres::PostgresSchemaOptions,
    pub(crate) embedded: EmbeddedSqlOptions,
    pub(crate) fail_unanalyzable: bool,
}

impl CompiledOptions {
    fn includes(&self, rel: &str, sql: bool) -> bool {
        let excluded = !self.exclude.is_empty() && self.exclude.is_match(rel);
        if excluded {
            return false;
        }
        if rel.ends_with(".sql") {
            return sql;
        }
        crate::codebase::dependencies::extract::is_indexable(Path::new(rel))
            && (self.include.is_empty() || self.include.is_match(rel))
    }
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
    sources: &std::sync::Arc<crate::codebase::ts_source::SourceStore>,
) -> Result<Vec<RuleFinding>> {
    let facts = crate::codebase::postgres::prepare_rule_sql_facts(
        root,
        all_files,
        std::sync::Arc::clone(sources),
        config,
        &[RULE_ID],
    )?;
    check_with_files_sources_and_facts(root, config, all_files, sources, &facts)
}

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
        .map(|rule| -> Result<super::PostgresFindings> {
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
            let sql_paths =
                crate::codebase::postgres::postgres_sql_paths(root, &files, &compiled.schema)?;
            let sql = crate::codebase::check_facts::PathMembership::new(&sql_paths);
            let files: Vec<PathBuf> = files
                .into_iter()
                .filter(|path| {
                    compiled.includes(&relative_slash_path(root, path), sql.contains(path))
                })
                .collect();
            scan(root, &compiled, &files, facts, sources)
        })
        .collect();
    let findings: super::PostgresFindings = all?.into_iter().flatten().collect();
    Ok(findings.finish())
}

fn compile_options(opts: &Options) -> Result<CompiledOptions> {
    let include = GlobMatcher::new(&opts.include, &format!("{RULE_ID} include"))?;
    let exclude = GlobMatcher::new(&opts.exclude, &format!("{RULE_ID} exclude"))?;
    GlobMatcher::new(&opts.sql_include, &format!("{RULE_ID} sqlInclude"))?;
    Ok(CompiledOptions {
        include,
        exclude,
        schema: crate::codebase::postgres::PostgresSchemaOptions {
            sql_include: opts.sql_include.clone(),
        },
        embedded: EmbeddedSqlOptions::configured(&opts.import_specifier, &opts.executor_names)
            .with_scoped_executors(&opts.executor_factory_names, &opts.executor_type_names)
            .with_trusted_sql_tags(&opts.trusted_sql_tags),
        fail_unanalyzable: fail_unanalyzable_sql(RULE_ID, &opts.unanalyzable_sql)?,
    })
}

#[cfg(test)]
mod sql_tests;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod followup_tests;
#[cfg(test)]
mod review_tests;
#[cfg(test)]
mod unanalyzable_tests;
