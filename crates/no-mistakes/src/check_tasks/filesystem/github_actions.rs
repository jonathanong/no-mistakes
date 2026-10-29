use anyhow::Result;
use no_mistakes::codebase::rules::{self, analysis_policy::AnalysisDiagnostics};
use no_mistakes::codebase::ts_source::SourceStore;
use no_mistakes::config::v2::NoMistakesConfig;
use std::borrow::Cow;
use std::path::{Path, PathBuf};

type WorkflowCheck =
    fn(&Path, &NoMistakesConfig, &[PathBuf], &SourceStore, bool) -> Result<AnalysisDiagnostics>;

const CHECKS: &[(&str, WorkflowCheck)] = &[
    (
        rules::github_actions_checkout_credentials::RULE_ID,
        rules::github_actions_checkout_credentials::diagnostics_with_files_and_sources,
    ),
    (
        rules::github_actions_job_permissions::RULE_ID,
        rules::github_actions_job_permissions::diagnostics_with_files_and_sources,
    ),
];

pub(super) fn run<'a>(
    root: &Path,
    config: &'a NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<(Cow<'a, NoMistakesConfig>, AnalysisDiagnostics)> {
    let mut remaining = Cow::Borrowed(config);
    let mut output = AnalysisDiagnostics::default();
    for &(id, check) in CHECKS {
        if config.rule_configured(id) {
            // Consume findings and advisories together, without a second file scan.
            output.append(check(root, config, files, sources, defer_suppression)?);
            remaining.to_mut().rules.retain(|rule| rule.rule != id);
        }
    }
    Ok((remaining, output))
}
