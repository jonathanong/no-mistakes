use super::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::{Path, PathBuf};

mod config;
mod delegate;
mod dispatch;
mod prepared;
pub(crate) use prepared::with_standalone as with_standalone_resolver;
#[doc(hidden)]
pub use prepared::PreparedConfigResolver;
mod extract;
mod options;
mod routes;
mod scan;
mod static_values;
mod words;

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
    if config.rule_applications(RULE_ID).is_empty() {
        return Ok(Vec::new());
    }
    prepared::with_standalone(root, config, all_files, sources, |resolver| {
        let diagnostics = dispatch::collect(
            root,
            config,
            all_files,
            sources,
            None,
            dispatch::CollectPolicy {
                require_tracked_snapshot: false,
                defer_suppression: true,
            },
            resolver,
        )?;
        Ok(dispatch::finish_advisories(diagnostics))
    })
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
    if config.rule_applications(RULE_ID).is_empty() {
        return Ok(Vec::new());
    }
    prepared::with_standalone(root, config, all_files, sources, |resolver| {
        Ok(dispatch::finish_findings(diagnostics_prepared(
            root, config, all_files, sources, snapshot, resolver, false,
        )?))
    })
}

#[doc(hidden)]
pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    snapshot: Option<&crate::codebase::ts_source::VisiblePathSnapshot>,
    defer_suppression: bool,
) -> Result<super::analysis_policy::AnalysisDiagnostics> {
    prepared::with_standalone(root, config, files, sources, |resolver| {
        diagnostics_prepared(
            root,
            config,
            files,
            sources,
            snapshot,
            resolver,
            defer_suppression,
        )
    })
}

#[doc(hidden)]
pub fn diagnostics_prepared(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    snapshot: Option<&crate::codebase::ts_source::VisiblePathSnapshot>,
    resolver: &PreparedConfigResolver<'_>,
    defer_suppression: bool,
) -> Result<super::analysis_policy::AnalysisDiagnostics> {
    let mut diagnostics = dispatch::collect(
        root,
        config,
        all_files,
        sources,
        snapshot,
        dispatch::CollectPolicy {
            require_tracked_snapshot: true,
            defer_suppression,
        },
        resolver,
    )?;
    if !defer_suppression {
        super::suppress_rule_findings_with_sources(root, &mut diagnostics.findings, sources);
        super::suppress_rule_findings_with_sources(root, &mut diagnostics.advisories, sources);
    }
    Ok(diagnostics)
}

pub(crate) fn check_prepared(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    snapshot: Option<&crate::codebase::ts_source::VisiblePathSnapshot>,
    resolver: &PreparedConfigResolver<'_>,
) -> Result<Vec<RuleFinding>> {
    let diagnostics =
        diagnostics_prepared(root, config, all_files, sources, snapshot, resolver, true)?;
    Ok(dispatch::finish_findings(diagnostics))
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod reexport_tests;
/// Configured Next.js project roots admitted to the ordinary project catalog.
#[doc(hidden)]
pub fn configured_project_roots(root: &Path, config: &NoMistakesConfig) -> Vec<PathBuf> {
    let mut roots = config
        .rule_applications(RULE_ID)
        .into_iter()
        .flat_map(|rule| super::target_roots(root, config, rule))
        .collect::<Vec<_>>();
    roots.sort();
    roots.dedup();
    roots
}
