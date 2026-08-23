use super::path_filter::GlobMatcher;
use super::RuleFinding;
use crate::codebase::postgres::EmbeddedSqlOptions;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod directive;
mod scan;

pub const RULE_ID: &str = "postgres-lock-ordering";

use directive::DEFAULT_SAFE_DIRECTIVE;
use scan::scan_with_sources;

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) include: Vec<String>,
    pub(crate) exclude: Vec<String>,
    pub(crate) import_specifier: String,
    pub(crate) executor_names: Vec<String>,
    pub(crate) safe_directive: String,
}

struct CompiledOptions {
    include: GlobMatcher,
    exclude: GlobMatcher,
    embedded: EmbeddedSqlOptions,
    safe_directive: String,
}

impl CompiledOptions {
    fn includes(&self, rel: &str) -> bool {
        (self.include.is_empty() || self.include.is_match(rel))
            && (self.exclude.is_empty() || !self.exclude.is_match(rel))
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
            let files: Vec<PathBuf> = files
                .into_iter()
                .filter(|path| compiled.includes(&relative_slash_path(root, path)))
                .collect();
            scan_with_sources(root, &compiled, &files, sources)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn compile_options(opts: &Options) -> Result<CompiledOptions> {
    let include = GlobMatcher::new(&opts.include, &format!("{RULE_ID} include"))?;
    let exclude = GlobMatcher::new(&opts.exclude, &format!("{RULE_ID} exclude"))?;
    let defaults = EmbeddedSqlOptions::default();
    Ok(CompiledOptions {
        include,
        exclude,
        embedded: EmbeddedSqlOptions {
            import_specifier: if opts.import_specifier.is_empty() {
                defaults.import_specifier
            } else {
                opts.import_specifier.clone()
            },
            executor_names: if opts.executor_names.is_empty() {
                defaults.executor_names
            } else {
                opts.executor_names.clone()
            },
        },
        safe_directive: if opts.safe_directive.is_empty() {
            DEFAULT_SAFE_DIRECTIVE.to_string()
        } else {
            opts.safe_directive.clone()
        },
    })
}

#[cfg(test)]
mod tests;
