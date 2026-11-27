use super::{analysis_policy::AnalysisDiagnostics};
use no_mistakes_rule_kernel::finding::RuleFinding;
use crate::config::v2::{NoMistakesConfig, Unanalyzable};
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod line;
mod scan;
mod workflow_step;
mod yaml;

pub const RULE_ID: &str = "github-actions-action-timeout-pair";

const DEFAULT_INCLUDE: &[&str] = &[
    ".github/workflows/**/*.yml",
    ".github/workflows/**/*.yaml",
    ".github/actions/**/action.yml",
    ".github/actions/**/action.yaml",
];

fn default_true() -> bool {
    true
}

#[derive(Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) include: Vec<String>,
    pub(crate) uses: Vec<String>,
    pub(crate) step_timeout_minutes: Option<u64>,
    pub(crate) nested_input: String,
    pub(crate) nested_timeout_seconds: Option<u64>,
    pub(crate) nested_below_step_timeout: bool,
    #[serde(default = "default_true")]
    pub(crate) forbid_nested_in_composite: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            include: Vec::new(),
            uses: Vec::new(),
            step_timeout_minutes: None,
            nested_input: String::new(),
            nested_timeout_seconds: None,
            nested_below_step_timeout: false,
            forbid_nested_in_composite: true,
        }
    }
}

pub(super) enum UsesSpec {
    Exact(String),
    Prefix(String),
}

pub(super) struct CompiledOptions {
    include: Vec<String>,
    uses: Vec<UsesSpec>,
    step_timeout_minutes: Option<u64>,
    nested_input: String,
    nested_timeout_seconds: Option<u64>,
    nested_below_step_timeout: bool,
    unanalyzable: Unanalyzable,
    forbid_nested_in_composite: bool,
    target_roots: Vec<PathBuf>,
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
    diagnostics_with_files_and_sources(root, config, all_files, sources, defer_suppression)
        .map(|output| output.findings)
}

#[doc(hidden)]
pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let mut diagnostics = AnalysisDiagnostics::default();
    for rule in config.rule_applications(RULE_ID) {
        let mut opts = compile_options(rule.try_rule_options()?);
        if opts.uses.is_empty() {
            continue;
        }
        if opts.nested_below_step_timeout {
            opts.unanalyzable = config.effective_rule_analysis(rule)?.unanalyzable;
        }
        opts.target_roots = super::target_roots(root, config, rule);
        let skip = super::skip_dir_set(config);
        let files: Vec<PathBuf> = all_files
            .iter()
            .filter(|path| {
                super::file_allowed_by_roots_and_skip(root, &skip, path, &opts.target_roots)
            })
            .cloned()
            .collect();
        let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
        if files.is_empty() {
            continue;
        }
        let files = super::matching_files(root, &opts.include, &files, &opts.target_roots)?;
        let mut application = scan_files(root, &opts, &files, sources, defer_suppression);
        if let Some(message) = rule
            .message
            .as_ref()
            .filter(|_| opts.nested_below_step_timeout)
        {
            for finding in application
                .findings
                .iter_mut()
                .chain(&mut application.advisories)
            {
                finding.message.clone_from(message);
            }
        }
        diagnostics.append(application);
    }
    Ok(super::file_diagnostics::sorted(diagnostics))
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
        uses: opts
            .uses
            .into_iter()
            .filter_map(|entry| {
                let normalized = yaml::normalize_uses(&entry);
                if normalized.is_empty() {
                    None
                } else if normalized.ends_with('@') {
                    Some(UsesSpec::Prefix(normalized.to_ascii_lowercase()))
                } else {
                    Some(UsesSpec::Exact(normalized))
                }
            })
            .collect(),
        step_timeout_minutes: opts.step_timeout_minutes,
        nested_input: opts.nested_input,
        nested_timeout_seconds: opts.nested_timeout_seconds,
        nested_below_step_timeout: opts.nested_below_step_timeout,
        unanalyzable: Unanalyzable::default(),
        forbid_nested_in_composite: opts.forbid_nested_in_composite,
        target_roots: Vec::new(),
    }
}

fn scan_files(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> AnalysisDiagnostics {
    files
        .par_iter()
        .map(|path| scan::check_file(root, path, opts, sources, defer_suppression))
        .reduce(AnalysisDiagnostics::default, |mut combined, per_file| {
            combined.append(per_file);
            combined
        })
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
