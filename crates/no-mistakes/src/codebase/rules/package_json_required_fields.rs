use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod scan;

pub const RULE_ID: &str = "package-json-required-fields";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) private: Option<bool>,
    #[serde(rename = "type")]
    pub(crate) type_value: Option<String>,
    pub(crate) license: Option<String>,
    pub(crate) require_scoped_name: bool,
    pub(crate) unscoped_name_exceptions: Vec<String>,
    pub(crate) main_when_file_exists: Option<String>,
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
        let (in_scope, _) =
            super::selected_files::rule_scope_files_with_roots(root, config, rule, all_files);
        let manifests = super::path_filter::filter_rule_files(root, config, rule, &in_scope)?;
        let policy = config.effective_rule_analysis(rule)?.unanalyzable;
        let mut findings = scan::scan(root, &opts, &manifests, &in_scope, sources);
        if !defer_suppression {
            super::suppress_rule_findings_with_sources(root, &mut findings, sources);
        }
        for finding in findings {
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

fn pkg_rel(root: &Path, path: &Path) -> String {
    relative_slash_path(root, path)
}

#[cfg(test)]
mod tests;
