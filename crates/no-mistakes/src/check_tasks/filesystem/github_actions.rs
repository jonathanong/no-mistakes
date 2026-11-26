use anyhow::Result;
use no_mistakes::codebase::rules::{self, analysis_policy::AnalysisDiagnostics};
use no_mistakes::codebase::ts_source::SourceStore;
use no_mistakes::config::v2::NoMistakesConfig;
use std::borrow::Cow;
use std::path::{Path, PathBuf};

mod topology;

type WorkflowCheck =
    fn(&Path, &NoMistakesConfig, &[PathBuf], &SourceStore, bool) -> Result<AnalysisDiagnostics>;

const CHECKS: &[(&str, WorkflowCheck)] = &[
    (
        rules::github_actions_action_timeout_pair::RULE_ID,
        rules::github_actions_action_timeout_pair::diagnostics_with_files_and_sources,
    ),
    (
        rules::github_actions_local_action_checkout::RULE_ID,
        rules::github_actions_local_action_checkout::diagnostics_with_files_and_sources,
    ),
    (
        rules::github_actions_no_required_pr_path_filters::RULE_ID,
        rules::github_actions_no_required_pr_path_filters::diagnostics_with_files_and_sources,
    ),
    (
        rules::github_actions_checkout_credentials::RULE_ID,
        rules::github_actions_checkout_credentials::diagnostics_with_files_and_sources,
    ),
    (
        rules::github_actions_job_permissions::RULE_ID,
        rules::github_actions_job_permissions::diagnostics_with_files_and_sources,
    ),
    (
        rules::github_actions_env_policy::RULE_ID,
        rules::github_actions_env_policy::diagnostics_with_files_and_sources,
    ),
    (
        rules::github_actions_no_insta_snapshot_updates::RULE_ID,
        rules::github_actions_no_insta_snapshot_updates::diagnostics_with_files_and_sources,
    ),
    (
        rules::github_actions_run_command_policy::RULE_ID,
        rules::github_actions_run_command_policy::diagnostics_with_files_and_sources,
    ),
    (
        rules::github_actions_pr_diff_range::RULE_ID,
        rules::github_actions_pr_diff_range::diagnostics_with_files_and_sources,
    ),
];

pub(super) fn run<'a>(
    root: &Path,
    config: &'a NoMistakesConfig,
    files: &[PathBuf],
    prepared: &rules::filesystem_dispatch::PreparedFilesystemRuleInputs<'_>,
    defer_suppression: bool,
) -> Result<(Cow<'a, NoMistakesConfig>, AnalysisDiagnostics)> {
    let mut output = AnalysisDiagnostics::default();
    let mut remaining = topology::run(root, config, prepared, defer_suppression, &mut output)?;
    for &(id, check) in CHECKS {
        if config.rule_configured(id) {
            // Consume findings and advisories together, without a second file scan.
            output.append(no_mistakes::perf_trace::filesystem_rule(id, || {
                check(root, config, files, &prepared.sources, defer_suppression)
            })?);
            super::handled::mark(remaining.to_mut(), |rule| rule == id);
        }
    }
    Ok((remaining, output))
}
