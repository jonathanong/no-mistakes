use super::RuleFinding;
use crate::codebase::postgres::PostgresSchemaOptions;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

mod redundancy;
mod scan;

pub const RULE_ID: &str = "postgres-redundant-index";
const DEFAULT_ALLOW_DIRECTIVE: &str = "redundant-index-allow";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) sql_include: Vec<String>,
    pub(crate) allow_directive: String,
    pub(crate) allowed_indexes: Vec<String>,
}

struct CompiledOptions {
    schema: PostgresSchemaOptions,
    allow_directive: String,
    allowed_indexes: BTreeSet<String>,
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
        let opts: Options = rule.rule_options();
        let compiled = compile_options(&opts);
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

fn compile_options(opts: &Options) -> CompiledOptions {
    CompiledOptions {
        schema: PostgresSchemaOptions {
            sql_include: if opts.sql_include.is_empty() {
                PostgresSchemaOptions::default().sql_include
            } else {
                opts.sql_include.clone()
            },
        },
        allow_directive: if opts.allow_directive.is_empty() {
            DEFAULT_ALLOW_DIRECTIVE.to_string()
        } else {
            opts.allow_directive.clone()
        },
        allowed_indexes: opts.allowed_indexes.iter().cloned().collect(),
    }
}

fn sql_rel(root: &Path, path: &Path) -> String {
    relative_slash_path(root, path)
}

#[cfg(test)]
mod tests;
