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
    defer_suppression: bool,
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
    super::check_with_borrowed_source_store(root, config, all_files, check_with_files_and_sources)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    check_with_files_and_sources_deferred(root, config, all_files, sources, false)
}

pub(crate) fn check_with_files_and_sources_deferred(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    super::run_selected_workflow_rule(
        super::SourcedRuleContext::new(root, config, all_files, sources),
        RULE_ID,
        |opts| {
            let mut compiled = compile_options(opts);
            compiled.defer_suppression = defer_suppression;
            compiled
        },
        |opts| &opts.include,
        scan_files,
    )
}

fn compile_options(opts: Options) -> CompiledOptions {
    let include = super::configured_includes(opts.include, DEFAULT_INCLUDE);
    let allowed_keys = if opts.allowed_keys.is_empty() {
        DEFAULT_ALLOWED_KEYS
            .iter()
            .map(|key| (*key).to_string())
            .collect()
    } else {
        opts.allowed_keys.into_iter().collect()
    };
    CompiledOptions {
        defer_suppression: false,
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
    crate::perf_trace::trace("github-actions-composite-step-schema.scan-files", || {
        files
            .par_iter()
            .flat_map(|path| check_file(root, path, opts, sources))
            .collect()
    })
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod deferred_tests;
