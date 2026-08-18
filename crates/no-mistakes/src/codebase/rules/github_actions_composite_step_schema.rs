use super::RuleFinding;
use crate::codebase::ts_source::discover_files;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

mod scan;
use scan::check_file;

pub const RULE_ID: &str = "github-actions-composite-step-schema";

const DEFAULT_INCLUDE: &[&str] = &[
    ".github/actions/**/action.yml",
    ".github/actions/**/action.yaml",
];

/// Composite-action step keys documented by GitHub.
///
/// `timeout-minutes` is intentionally absent: GitHub does not support it on
/// composite-action steps.
const DEFAULT_ALLOWED_KEYS: &[&str] = &[
    "name",
    "id",
    "if",
    "uses",
    "run",
    "shell",
    "with",
    "env",
    "working-directory",
    "continue-on-error",
];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) include: Vec<String>,
    pub(crate) allowed_keys: Vec<String>,
    pub(crate) extra_forbidden_keys: Vec<String>,
}

struct CompiledOptions {
    include: Vec<String>,
    allowed_keys: HashSet<String>,
    extra_forbidden_keys: HashSet<String>,
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let files = discover_files(&root, &config.filesystem.skip_directories);
    check_with_files(&root, config, &files)
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
        let opts = compile_options(rule.rule_options());
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
        findings.extend(scan_files(root, &opts, &files, sources));
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
    let allowed_keys = if opts.allowed_keys.is_empty() {
        DEFAULT_ALLOWED_KEYS
            .iter()
            .map(|key| (*key).to_string())
            .collect()
    } else {
        opts.allowed_keys.into_iter().collect()
    };
    CompiledOptions {
        include,
        allowed_keys,
        extra_forbidden_keys: opts.extra_forbidden_keys.into_iter().collect(),
    }
}

fn scan_files(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    files
        .par_iter()
        .flat_map(|path| check_file(root, path, opts, sources))
        .collect()
}

#[cfg(test)]
mod tests;
