use super::RuleFinding;
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod diagnostics;
mod locations;
mod scan;

pub const RULE_ID: &str = "github-actions-local-action-checkout";

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Options {}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    super::check_sourced_standalone(root, config, check_with_files)
}

super::diagnostic_source_rule_entrypoints!();

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
