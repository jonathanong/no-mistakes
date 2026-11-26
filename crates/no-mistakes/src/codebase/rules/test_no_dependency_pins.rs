mod compile;
mod patterns;
mod runtime_patterns;
mod scan;

use super::analysis_policy::AnalysisDiagnostics;
use super::file_diagnostics::{classified_applications, ClassifiedInputFindings, FileRuleInputs};
use super::path_filter::GlobMatcher;
use super::source_access::read_failure;
use super::RuleFinding;
use crate::codebase::ts_source::{discover_files, relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use compile::{compile_builtin, compile_pattern};
use patterns::DEFAULT_PATTERNS;
use rayon::prelude::*;
use regex::Regex;
use runtime_patterns::RUNTIME_PATTERNS;
use scan::check_source;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "test-no-dependency-pins";

/// the supported test-file pattern.
const DEFAULT_INCLUDE_RE: &str =
    r"(?:^|/)(?:__tests__/.*|[^/]+(?:\.mock)?\.test\.(?:mts|ts|tsx|mjs|js|cts|cjs))$";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) include: Vec<String>,
    pub(crate) patterns: Vec<PatternOption>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub(crate) struct PatternOption {
    pub(crate) reason: String,
    pub(crate) regex: String,
}

struct CompiledPattern {
    reason: String,
    regex: Regex,
    reject_preceding_at: bool,
    multiline: bool,
    /// Default patterns report their `pin` capture, skip placeholder values,
    /// and honor `line_context`; custom patterns report every match.
    builtin: bool,
    line_context: Option<Regex>,
}

struct CompiledOptions {
    include: GlobMatcher,
    default_include: Regex,
    patterns: Vec<CompiledPattern>,
}

impl CompiledOptions {
    fn includes(&self, rel: &str) -> bool {
        if self.include.is_empty() {
            self.default_include.is_match(rel)
        } else {
            self.include.is_match(rel)
        }
    }
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let files = discover_files(root, &config.filesystem.skip_directories);
    check_with_files(root, config, &files)
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(all_files);
    diagnostics_with_files_and_sources(root, config, all_files, &sources, true)
        .map(|output| output.findings)
}

#[doc(hidden)]
pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    classified_applications(
        FileRuleInputs {
            root,
            config,
            files: all_files,
            sources,
            defer_suppression,
        },
        RULE_ID,
        |opts, files, _, _| {
            Ok(ClassifiedInputFindings {
                findings: scan_with_sources(root, opts, files, sources)?,
                ..Default::default()
            })
        },
    )
}

fn scan_with_sources(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    let compiled = compile_options(opts)?;
    let mut findings: Vec<RuleFinding> = files
        .par_iter()
        .flat_map(|path| check_file_with_sources(root, path, &compiled, sources))
        .collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn compile_options(opts: &Options) -> Result<CompiledOptions> {
    let include = GlobMatcher::new(&opts.include, &format!("{RULE_ID} include"))?;
    let patterns = if opts.patterns.is_empty() {
        let dependency = DEFAULT_PATTERNS
            .iter()
            .map(|(reason, regex, multiline)| compile_builtin(reason, regex, *multiline, None));
        let runtime = RUNTIME_PATTERNS
            .iter()
            .map(|(reason, regex, context)| compile_builtin(reason, regex, false, *context));
        dependency.chain(runtime).collect::<Result<Vec<_>>>()?
    } else {
        opts.patterns
            .iter()
            .map(|pattern| compile_pattern(&pattern.reason, &pattern.regex, false))
            .collect::<Result<Vec<_>>>()?
    };
    Ok(CompiledOptions {
        include,
        default_include: default_include_regex(),
        patterns,
    })
}

fn default_include_regex() -> Regex {
    Regex::new(DEFAULT_INCLUDE_RE).expect("default test-file include regex is valid")
}

fn check_file_with_sources(
    root: &Path,
    path: &Path,
    opts: &CompiledOptions,
    sources: &SourceStore,
) -> Vec<RuleFinding> {
    let rel = relative_slash_path(root, path);
    if !opts.includes(&rel) {
        return Vec::new();
    }
    match sources.read_source_path(path) {
        Ok(content) => check_source(&rel, &content, opts),
        Err(error) => vec![read_failure(root, path, RULE_ID, &error)],
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
