use super::RuleFinding;
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod diagnostics;
mod findings;
mod permissions;

pub const RULE_ID: &str = "github-actions-job-permissions";

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Options {}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    super::check_sourced_standalone(root, config, |root, config, files| {
        super::check_with_borrowed_source_store(
            root,
            config,
            files,
            |root, config, files, sources| {
                diagnostics_with_files_and_sources(root, config, files, sources, false)
                    .map(|output| output.findings)
            },
        )
    })
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    super::check_with_borrowed_source_store(root, config, files, check_with_files_and_sources)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    diagnostics_with_files_and_sources(root, config, files, sources, true)
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

#[cfg(test)]
mod tests;
