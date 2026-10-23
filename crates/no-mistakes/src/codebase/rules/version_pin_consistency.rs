use super::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod key;
mod parse;
mod path;
mod scan;
mod source;

pub const RULE_ID: &str = "version-pin-consistency";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) source_file: String,
    pub(crate) source_key: String,
    pub(crate) anchors: Vec<Anchor>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Anchor {
    pub(crate) file: String,
    pub(crate) pattern: String,
    pub(crate) label: String,
}

super::borrowed_source_rule_entrypoints!();

pub(crate) fn check_with_files_sources_and_deferred_suppression(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    diagnostics_with_files_and_sources(root, config, all_files, sources, defer_suppression)
        .map(|diagnostics| diagnostics.findings)
}

#[doc(hidden)]
pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Result<super::analysis_policy::AnalysisDiagnostics> {
    let mut diagnostics = super::analysis_policy::AnalysisDiagnostics::default();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.try_rule_options()?;
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
        let policy = config.effective_rule_analysis(rule)?.unanalyzable;
        for finding in scan::scan(root, &opts, &files, sources, defer_suppression) {
            if finding.target.as_deref() == Some("unanalyzable") {
                diagnostics.unanalyzable(policy, finding);
            } else {
                diagnostics.findings.push(finding);
            }
        }
    }
    super::sort_findings(&mut diagnostics.findings);
    super::sort_findings(&mut diagnostics.advisories);
    Ok(diagnostics)
}

#[cfg(test)]
mod tests;
