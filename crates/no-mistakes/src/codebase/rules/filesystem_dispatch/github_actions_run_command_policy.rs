use crate::codebase::rules::{self, RuleFinding};
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use no_mistakes_rule_kernel::file_diagnostics::AnalysisDiagnostics;
use std::path::{Path, PathBuf};

mod diagnostics;
mod judge;
mod matcher;
pub(in crate::codebase::rules) mod options;
mod runs;

pub const RULE_ID: &str = "github-actions-run-command-policy";

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    rules::check_sourced_standalone(root, config, check_with_files)
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    rules::check_with_borrowed_source_store(root, config, all_files, check_with_files_and_sources)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    check_with_files_sources_and_deferred_suppression(root, config, all_files, sources, false)
}

pub(crate) fn check_with_files_sources_and_deferred_suppression(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    diagnostics_with_files_and_sources(root, config, files, sources, defer_suppression)
        .map(|output| output.findings)
}

pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    diagnostics::run(root, config, files, sources, defer_suppression)
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
