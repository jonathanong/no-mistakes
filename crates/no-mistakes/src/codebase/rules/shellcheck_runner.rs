use no_mistakes_rule_kernel::finding::RuleFinding;
use crate::codebase::ts_source::{SourceStore, VisiblePathSnapshot};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "shellcheck-runner";

mod candidates;
mod diagnostics;
mod execution;
pub use diagnostics::diagnostics_with_files_sources_and_snapshot;
use execution::run_shellcheck;
mod options;
pub(crate) use options::Options;
mod selection;

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let rules = config.rule_applications(RULE_ID);
    if rules.is_empty() {
        return Ok(Vec::new());
    }
    let snapshot = VisiblePathSnapshot::new(root);
    let mut files: Vec<_> = rules
        .into_iter()
        .flat_map(|rule| super::target_roots(root, config, rule))
        .flat_map(|target| snapshot.paths_for(&target).as_ref().clone())
        .collect();
    files.sort();
    files.dedup();
    let sources = super::source_store_for_files(&files);
    Ok(diagnostics_with_files_sources_and_snapshot(
        root, config, &files, &sources, &snapshot, false,
    )?
    .findings)
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
    let snapshot = VisiblePathSnapshot::from_paths(root, all_files);
    check_with_files_sources_and_snapshot(root, config, all_files, sources, &snapshot)
}

pub(crate) fn check_with_files_sources_and_snapshot(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
    snapshot: &VisiblePathSnapshot,
) -> Result<Vec<RuleFinding>> {
    Ok(diagnostics_with_files_sources_and_snapshot(
        root, config, all_files, sources, snapshot, true,
    )?
    .findings)
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
