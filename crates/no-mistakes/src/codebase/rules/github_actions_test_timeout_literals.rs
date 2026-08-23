use super::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

mod scan;

pub const RULE_ID: &str = "github-actions-test-timeout-literals";

const DEFAULT_INCLUDE: &[&str] = &[
    ".github/workflows/**/*.test.mts",
    ".github/workflows/**/*.test.ts",
];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) include: Vec<String>,
    pub(crate) allow: Vec<AllowEntry>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct AllowEntry {
    pub(crate) file: String,
    pub(crate) text: String,
    pub(crate) reason: String,
}

struct CompiledOptions {
    include: Vec<String>,
    allow: BTreeMap<String, String>,
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(all_files);
    check_with_files_and_sources(root, config, all_files, &sources, false)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts = compile_options(rule.rule_options()?);
        let target_roots = super::target_roots(root, config, rule);
        let skip = super::skip_dir_set(config);
        let files: Vec<PathBuf> = all_files
            .iter()
            .filter(|path| super::file_allowed_by_roots_and_skip(root, &skip, path, &target_roots))
            .cloned()
            .collect();
        let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
        if files.is_empty() {
            continue;
        }
        let files = super::matching_files(root, &opts.include, &files, &target_roots)?;
        findings.extend(scan_files(root, &opts, &files, sources, defer_suppression));
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn compile_options(opts: Options) -> CompiledOptions {
    let include = if opts.include.is_empty() {
        DEFAULT_INCLUDE
            .iter()
            .map(|pattern| (*pattern).to_string())
            .collect()
    } else {
        opts.include
    };
    CompiledOptions {
        include,
        allow: opts
            .allow
            .into_iter()
            .map(|entry| {
                (
                    format!("{}#{}", entry.file.trim(), entry.text.trim()),
                    entry.reason,
                )
            })
            .collect(),
    }
}

fn scan_files(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Vec<RuleFinding> {
    files
        .par_iter()
        .flat_map(|path| scan::check_file(root, path, opts, sources, defer_suppression))
        .collect()
}

#[cfg(test)]
mod tests;
