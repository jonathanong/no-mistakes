use super::super::*;
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

pub(super) fn run(
    rule_id: &str,
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Option<Result<Vec<RuleFinding>>> {
    Some(match rule_id {
        INTEGRATION_TEST_NO_MOCKS => {
            integration_test_no_mocks::check_with_files_and_sources(root, config, files, sources)
        }
        MARKDOWN_LINK_DISPLAY_TEXT => {
            markdown_link_display_text::check_with_files_and_sources(root, config, files, sources)
        }
        MARKDOWN_EVAL_TESTS => {
            markdown_eval_tests::check_with_files_and_sources(root, config, files, sources)
        }
        STRUCTURED_CONFIG_POLICY => {
            structured_config_policy::check_with_files_and_sources(root, config, files, sources)
        }
        CONFIG_PATH_REFERENCES => {
            config_path_references::check_with_files_and_sources(root, config, files, sources)
        }
        GITHUB_ACTIONS_ACTION_TIMEOUT_PAIR => {
            github_actions_action_timeout_pair::check_with_files_and_sources(
                root,
                config,
                files,
                sources,
                defer_suppression,
            )
        }
        GITHUB_ACTIONS_COMPOSITE_STEP_SCHEMA => {
            github_actions_composite_step_schema::check_with_files_and_sources(
                root, config, files, sources,
            )
        }
        GITHUB_ACTIONS_JOB_TIMEOUTS => {
            github_actions_job_timeouts::check_with_files_and_sources(root, config, files, sources)
        }
        GITHUB_ACTIONS_TEST_TIMEOUT_LITERALS => {
            github_actions_test_timeout_literals::check_with_files_and_sources(
                root,
                config,
                files,
                sources,
                defer_suppression,
            )
        }
        GITHUB_ACTIONS_PINNED_HASH => {
            github_actions_pinned_hash::check_with_files_and_sources(root, config, files, sources)
        }
        TSCONFIG_FILE_COVERAGE => {
            tsconfig_file_coverage::check_with_files_and_sources(root, config, files, sources)
        }
        NO_EMPTY_OR_COMMENTS_ONLY_FILES => {
            no_empty_or_comments_only_files::check_with_files_and_sources(
                root, config, files, sources,
            )
        }
        NEXTJS_REDIRECT_DESTINATIONS => {
            nextjs_redirect_destinations::check_with_files_and_sources(root, config, files, sources)
        }
        _ => return None,
    })
}
