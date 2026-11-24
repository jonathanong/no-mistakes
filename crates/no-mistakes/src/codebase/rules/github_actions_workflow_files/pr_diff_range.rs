use crate::codebase::rules::{self, RuleFinding};
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod diff;
mod exprs;
mod runs;
mod yaml_shapes;

pub const RULE_ID: &str = "github-actions-pr-diff-range";

/// The rule has no options. A non-empty options map is rejected.
#[derive(Debug, Default, Deserialize)]
struct Options;

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
    rules::run_sourced_file_rule(root, config, all_files, sources, RULE_ID, scan)
}

fn scan(
    root: &Path,
    _opts: &Options,
    files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    super::scan_files(root, files, sources, &|rel, content| {
        runs::check_source(rel, content)
    })
}

#[cfg(test)]
mod tests;
