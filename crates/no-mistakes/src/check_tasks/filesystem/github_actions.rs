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
        rules::github_actions_cache_setup_policy::RULE_ID,
        rules::github_actions_cache_setup_policy::diagnostics_with_files_and_sources,
    ),
    (
        rules::github_actions_cache_key_policy::RULE_ID,
        rules::github_actions_cache_key_policy::diagnostics_with_files_and_sources,
    ),
    (
        rules::github_actions_action_timeout_pair::RULE_ID,
        rules::github_actions_action_timeout_pair::diagnostics_with_files_and_sources,
    ),
    (
        rules::github_actions_docker_host_ports::RULE_ID,
        rules::github_actions_docker_host_ports::diagnostics_with_files_and_sources,
    ),
    (
        rules::github_actions_no_poll_wait_job::RULE_ID,
        rules::github_actions_no_poll_wait_job::diagnostics_with_files_and_sources,
    ),
    (
        rules::github_actions_vitest_project_selection::RULE_ID,
        rules::github_actions_vitest_project_selection::diagnostics_with_files_and_sources,
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
        rules::github_actions_workflow_script_paths::RULE_ID,
        rules::github_actions_workflow_script_paths::diagnostics_with_files_and_sources,
    ),
    (
        rules::github_actions_step_timeouts::RULE_ID,
        rules::github_actions_step_timeouts::diagnostics_with_files_and_sources,
    ),
    (
        rules::github_actions_job_timeouts::RULE_ID,
        job_timeouts_diagnostics,
    ),
    (
        rules::github_actions_pr_diff_range::RULE_ID,
        rules::github_actions_pr_diff_range::diagnostics_with_files_and_sources,
    ),
];

fn job_timeouts_diagnostics(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    Ok(AnalysisDiagnostics {
        findings: rules::github_actions_job_timeouts::check_with_files_and_sources_deferred(
            root,
            config,
            files,
            sources,
            defer_suppression,
        )?,
        ..AnalysisDiagnostics::default()
    })
}

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
            let result = no_mistakes::perf_trace::filesystem_rule(id, || {
                check(root, config, files, &prepared.sources, defer_suppression)
            });
            let file_rule_session = prepared.session.filter(|_| {
                matches!(
                    id,
                    rules::github_actions_docker_host_ports::RULE_ID
                        | rules::GITHUB_ACTIONS_NO_POLL_WAIT_JOB
                        | rules::GITHUB_ACTIONS_PR_DIFF_RANGE
                        | rules::github_actions_vitest_project_selection::RULE_ID
                )
            });
            let diagnostics = match file_rule_session {
                Some(session) => session
                    .capture_check_result(id, result)?
                    .unwrap_or_default(),
                None => result?,
            };
            output.append(diagnostics);
            super::handled::mark(remaining.to_mut(), |rule| rule == id);
        }
    }
    Ok((remaining, output))
}

#[cfg(test)]
mod tests;
