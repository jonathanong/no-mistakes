use super::RuleFinding;
use crate::codebase::postgres::PostgresSchemaOptions;
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod scan;

pub const RULE_ID: &str = "postgres-identifier-length";

#[derive(Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) sql_include: Vec<String>,
    pub(crate) max_bytes: i64,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            sql_include: Vec::new(),
            max_bytes: default_max_bytes(),
        }
    }
}

fn default_max_bytes() -> i64 {
    63
}

struct CompiledOptions {
    schema: PostgresSchemaOptions,
    max_bytes: usize,
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
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.try_rule_options()?;
        let compiled = compile_options(&opts)?;
        let target_roots = super::target_roots(root, config, rule);
        let skip = super::skip_dir_set(config);
        let files: Vec<PathBuf> = all_files
            .iter()
            .filter(|path| super::file_allowed_by_roots_and_skip(root, &skip, path, &target_roots))
            .cloned()
            .collect();
        let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
        findings.extend(scan::scan(root, &compiled, &files, sources)?);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn compile_options(opts: &Options) -> Result<CompiledOptions> {
    if !(1..=63).contains(&opts.max_bytes) {
        bail!("{RULE_ID} option maxBytes: must be from 1 through 63");
    }
    Ok(CompiledOptions {
        schema: PostgresSchemaOptions {
            sql_include: if opts.sql_include.is_empty() {
                PostgresSchemaOptions::default().sql_include
            } else {
                opts.sql_include.clone()
            },
        },
        max_bytes: opts.max_bytes as usize,
    })
}

#[cfg(test)]
mod tests;
