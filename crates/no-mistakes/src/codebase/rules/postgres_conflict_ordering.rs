use super::path_filter::GlobMatcher;
use super::RuleFinding;
use crate::codebase::postgres::{
    fail_unanalyzable_sql, postgres_sql_paths, EmbeddedSqlOptions, PostgresSchemaOptions,
};
use crate::codebase::rules::postgres_lock_ordering::directive::DEFAULT_SAFE_DIRECTIVE;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod scan;

pub const RULE_ID: &str = "postgres-conflict-ordering";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) include: Vec<String>,
    pub(crate) exclude: Vec<String>,
    pub(crate) import_specifier: String,
    pub(crate) executor_names: Vec<String>,
    pub(crate) schema_catalog_path: String,
    pub(crate) sql_include: Vec<String>,
    pub(crate) unanalyzable_sql: String,
    pub(crate) safe_directive: String,
}

pub(crate) struct CompiledOptions {
    include: GlobMatcher,
    exclude: GlobMatcher,
    embedded: EmbeddedSqlOptions,
    schema_catalog_path: String,
    sql_sources: Option<PostgresSchemaOptions>,
    fail_unanalyzable: bool,
    safe_directive: String,
}

impl CompiledOptions {
    fn includes(&self, rel: &str) -> bool {
        (self.include.is_empty() || self.include.is_match(rel))
            && (self.exclude.is_empty() || !self.exclude.is_match(rel))
    }

    fn excludes(&self, rel: &str) -> bool {
        !self.exclude.is_empty() && self.exclude.is_match(rel)
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
    let profiles = crate::codebase::postgres::configured_embedded_sql_options(config, &[RULE_ID])?;
    let catalog_paths =
        crate::codebase::postgres::configured_schema_catalog_paths(config, &[RULE_ID])?;
    let facts = crate::codebase::postgres::prepare_embedded_sql_facts(
        root,
        all_files,
        std::sync::Arc::clone(sources),
        profiles,
        catalog_paths,
    );
    check_with_files_sources_and_facts(root, config, all_files, sources, &facts)
}

pub(crate) fn check_with_files_sources_and_facts(
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
            let scoped_files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            let mut files: Vec<PathBuf> = scoped_files
                .iter()
                .filter(|path| compiled.includes(&relative_slash_path(root, path)))
                .cloned()
                .collect();
            if let Some(sql_sources) = &compiled.sql_sources {
                files.extend(
                    postgres_sql_paths(root, &scoped_files, sql_sources)?
                        .into_iter()
                        .filter(|path| !compiled.excludes(&relative_slash_path(root, path))),
                );
            }
            files.sort();
            files.dedup();
            scan::scan_with_sources(root, &compiled, &files, sources, facts)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn compile_options(opts: &Options) -> Result<CompiledOptions> {
    if opts.schema_catalog_path.is_empty() {
        bail!("{RULE_ID} requires options.schemaCatalogPath");
    }
    let include = GlobMatcher::new(&opts.include, &format!("{RULE_ID} include"))?;
    let exclude = GlobMatcher::new(&opts.exclude, &format!("{RULE_ID} exclude"))?;
    Ok(CompiledOptions {
        include,
        exclude,
        embedded: EmbeddedSqlOptions::configured(&opts.import_specifier, &opts.executor_names),
        schema_catalog_path: opts.schema_catalog_path.clone(),
        sql_sources: (!opts.sql_include.is_empty()).then(|| PostgresSchemaOptions {
            sql_include: opts.sql_include.clone(),
        }),
        fail_unanalyzable: fail_unanalyzable_sql(RULE_ID, &opts.unanalyzable_sql)?,
        safe_directive: if opts.safe_directive.is_empty() {
            DEFAULT_SAFE_DIRECTIVE.to_string()
        } else {
            opts.safe_directive.clone()
        },
    })
}

#[cfg(test)]
mod compile_coverage_tests;
#[cfg(test)]
mod tests;
