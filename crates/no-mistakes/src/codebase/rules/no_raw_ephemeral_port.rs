use super::path_filter::GlobMatcher;
use super::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod ast;
mod python;
mod scan;

pub const RULE_ID: &str = "no-raw-ephemeral-port";

pub(super) const DEFAULT_MESSAGE: &str =
    "raw ephemeral port 0 bind/listen can occupy a deterministic runner slice";

const DEFAULT_INCLUDE: &[&str] = &[
    "**/*.bash",
    "**/*.cjs",
    "**/*.cts",
    "**/*.js",
    "**/*.jsx",
    "**/*.mjs",
    "**/*.mts",
    "**/*.py",
    "**/*.sh",
    "**/*.ts",
    "**/*.tsx",
    "**/*.yaml",
    "**/*.yml",
    "**/*.zsh",
];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) include: Vec<String>,
    pub(crate) allow: Vec<String>,
    pub(crate) message: Option<String>,
}

pub(super) struct CompiledOptions {
    include: Vec<String>,
    allow: GlobMatcher,
    bind: regex::Regex,
    message: String,
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
    check_with_files_sources_and_deferred_suppression(root, config, all_files, sources, false)
}

pub(crate) fn check_with_files_sources_and_deferred_suppression(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts = compile_options(rule.rule_options()?)?;
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

fn compile_options(opts: Options) -> Result<CompiledOptions> {
    let include = if opts.include.is_empty() {
        DEFAULT_INCLUDE
            .iter()
            .map(|pattern| (*pattern).to_string())
            .collect()
    } else {
        opts.include
    };
    Ok(CompiledOptions {
        include,
        allow: GlobMatcher::new(&opts.allow, &format!("{RULE_ID} allow"))?,
        bind: regex::Regex::new(python::BIND_PATTERN).expect("bind pattern"),
        message: match opts.message.filter(|hint| !hint.is_empty()) {
            Some(hint) => format!("{DEFAULT_MESSAGE} {hint}"),
            None => DEFAULT_MESSAGE.to_string(),
        },
    })
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
