use super::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::{Path, PathBuf};

mod config;
mod dispatch;
mod extract;
mod options;
mod routes;
mod scan;
mod static_values;

pub const RULE_ID: &str = "nextjs-redirect-destinations";

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(root);
    let files = crate::codebase::ts_source::discover_files_from_visible(
        root,
        &config.filesystem.skip_directories,
        &snapshot.paths_for(root),
    );
    let sources = snapshot.source_store_for(root);
    check_with_files_sources_and_snapshot(root, config, &files, &sources, Some(&snapshot))
}

pub fn advisories_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(all_files);
    advisories_with_files_and_sources(root, config, all_files, &sources)
}

pub fn advisories_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    advisories_with_files_sources_and_deferred_suppression(root, config, all_files, sources).map(
        |advisories| {
            let mut advisories = advisories;
            super::suppress_rule_findings_with_sources(root, &mut advisories, sources);
            advisories
        },
    )
}

#[doc(hidden)]
pub fn advisories_with_files_sources_and_deferred_suppression(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let diagnostics = dispatch::collect(root, config, all_files, sources, None, false)?;
    Ok(dispatch::finish_advisories(diagnostics))
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
    check_with_files_sources_and_snapshot(root, config, all_files, sources, None)
}

pub(crate) fn check_with_files_sources_and_snapshot(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    snapshot: Option<&crate::codebase::ts_source::VisiblePathSnapshot>,
) -> Result<Vec<RuleFinding>> {
    let diagnostics = dispatch::collect(root, config, all_files, sources, snapshot, true)?;
    Ok(dispatch::finish_findings(diagnostics))
}

#[cfg(test)]
mod tests;
