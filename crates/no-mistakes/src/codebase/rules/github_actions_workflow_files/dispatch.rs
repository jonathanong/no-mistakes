use super::{env_policy, no_poll_wait_job, pr_diff_range};
use crate::codebase::rules::{
    filesystem_dispatch::github_actions_workflow_script_paths,
    github_actions_cache_setup_policy, github_actions_no_insta_snapshot_updates,
    github_actions_no_required_pr_path_filters,
};
use no_mistakes_rule_kernel::finding::RuleFinding;
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Run a workflow rule handled by this dispatcher.
pub(in crate::codebase::rules) fn run(
    rule_id: &str,
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Option<Result<Vec<RuleFinding>>> {
    match rule_id {
        github_actions_cache_setup_policy::RULE_ID => Some(
            github_actions_cache_setup_policy::check_with_files_sources_and_deferred_suppression(
                root,
                config,
                files,
                sources,
                defer_suppression,
            ),
        ),
        env_policy::RULE_ID => Some(env_policy::check_with_files_and_sources(
            root,
            config,
            files,
            sources,
            defer_suppression,
        )),
        github_actions_no_insta_snapshot_updates::RULE_ID => Some(
            github_actions_no_insta_snapshot_updates::check_with_files_and_sources(
                root,
                config,
                files,
                sources,
                defer_suppression,
            ),
        ),
        github_actions_workflow_script_paths::RULE_ID => Some(
            github_actions_workflow_script_paths::check_with_files_sources_and_deferred_suppression(
                root,
                config,
                files,
                sources,
                defer_suppression,
            ),
        ),
        no_poll_wait_job::RULE_ID => Some(no_poll_wait_job::check_with_files_and_sources(
            root, config, files, sources,
        )),
        pr_diff_range::RULE_ID => Some(pr_diff_range::check_with_files_and_sources(
            root,
            config,
            files,
            sources,
            defer_suppression,
        )),
        github_actions_no_required_pr_path_filters::RULE_ID => Some(
            github_actions_no_required_pr_path_filters::check_with_files_sources_and_deferred_suppression(
                root, config, files, sources, defer_suppression,
            ),
        ),
        _ => None,
    }
}
