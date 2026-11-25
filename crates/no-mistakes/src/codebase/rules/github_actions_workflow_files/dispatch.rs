use super::{env_policy, no_poll_wait_job, pr_diff_range};
use crate::codebase::rules::{github_actions_no_insta_snapshot_updates, RuleFinding};
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Run a workflow-script rule by id; `None` means the id is not one of them.
pub(in crate::codebase::rules) fn run(
    rule_id: &str,
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Option<Result<Vec<RuleFinding>>> {
    match rule_id {
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
        _ => None,
    }
}
