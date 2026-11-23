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
        NO_MISTAKES_CONFIG => {
            no_mistakes_config::check_with_files_and_sources(root, config, files, sources)
        }
        INTEGRATION_TEST_NO_MOCKS => {
            integration_test_no_mocks::check_with_files_and_sources(root, config, files, sources)
        }
        MARKDOWN_LINK_DISPLAY_TEXT => {
            markdown_link_display_text::check_with_files_and_sources(root, config, files, sources)
        }
        MARKDOWN_EVAL_TESTS => {
            markdown_eval_tests::check_with_files_and_sources(root, config, files, sources)
        }
        DECLARED_PAYLOAD_COMPATIBILITY => {
            declared_payload_compatibility::check_with_files_and_sources(
                root, config, files, sources,
            )
        }
        STRUCTURED_CONFIG_POLICY => {
            structured_config_policy::check_with_files_and_sources(root, config, files, sources)
        }
        CONFIG_PATH_REFERENCES => {
            config_path_references::check_with_files_and_sources(root, config, files, sources)
        }
        GITHUB_ACTIONS_ACTION_INPUT_POLICY => {
            github_actions_action_input_policy::check_with_files_and_sources(
                root,
                config,
                files,
                sources,
                defer_suppression,
            )
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
            github_actions_composite_step_schema::check_with_files_and_sources_deferred(
                root,
                config,
                files,
                sources,
                defer_suppression,
            )
        }
        GITHUB_ACTIONS_JOB_TIMEOUTS => {
            github_actions_job_timeouts::check_with_files_and_sources_deferred(
                root,
                config,
                files,
                sources,
                defer_suppression,
            )
        }
        GITHUB_ACTIONS_STEP_TIMEOUTS => github_actions_step_timeouts::check_with_files_and_sources(
            root,
            config,
            files,
            sources,
            defer_suppression,
        ),
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
        GITHUB_ACTIONS_RUN_COMMAND_POLICY => {
            github_actions_run_command_policy::check_with_files_sources_and_deferred_suppression(
                root,
                config,
                files,
                sources,
                defer_suppression,
            )
        }
        GITHUB_ACTIONS_CACHE_KEY_POLICY => {
            github_actions_cache_key_policy::check_with_files_sources_and_deferred_suppression(
                root, config, files, sources, defer_suppression,
            )
        }
        GITHUB_ACTIONS_MAX_INLINE_SCRIPT_LINES => {
            github_actions_max_inline_script_lines::check_with_files_sources_and_deferred_suppression(
                root, config, files, sources, defer_suppression,
            )
        }
        GITHUB_ACTIONS_SCRIPT_EXPRESSIONS => {
            github_actions_script_expressions::check_with_files_and_sources(
                root, config, files, sources,
            )
        }
        GITHUB_ACTIONS_CHECKOUT_CREDENTIALS => {
            github_actions_checkout_credentials::check_with_files_and_sources(
                root, config, files, sources,
            )
        }
        GITHUB_ACTIONS_LOCAL_ACTION_CHECKOUT => {
            github_actions_local_action_checkout::check_with_files_sources_and_deferred_suppression(root, config, files, sources, defer_suppression)
        }
        GITHUB_ACTIONS_JOB_PERMISSIONS => {
            github_actions_job_permissions::check_with_files_and_sources(
                root, config, files, sources,
            )
        }
        GITHUB_ACTIONS_DOCKER_HOST_PORTS => {
            github_actions_docker_host_ports::check_with_files_and_sources(
                root, config, files, sources,
            )
        }
        GITHUB_ACTIONS_NO_POLL_WAIT_JOB => {
            no_poll_wait_job::check_with_files_and_sources(root, config, files, sources)
        }
        GITHUB_ACTIONS_PR_DIFF_RANGE => {
            pr_diff_range::check_with_files_and_sources(root, config, files, sources)
        }
        GITHUB_ACTIONS_VITEST_PROJECT_SELECTION => {
            github_actions_vitest_project_selection::check_with_files_and_sources(
                root, config, files, sources,
            )
        }
        TSCONFIG_FILE_COVERAGE => {
            tsconfig_file_coverage::check_with_files_and_sources(root, config, files, sources)
        }
        VITE_OPTIMIZE_DEPS_NESTED_IMPORTS => {
            vite_optimize_deps_nested_imports::check_with_files_sources_and_deferred_suppression(
                root,
                config,
                files,
                sources,
                defer_suppression,
            )
        }
        TEST_FIXTURE_SIZE_CAP => test_fixture_size_cap::check_with_files_and_sources(
            root,
            config,
            files,
            sources,
            defer_suppression,
        ),
        NO_EMPTY_OR_COMMENTS_ONLY_FILES => {
            no_empty_or_comments_only_files::check_with_files_and_sources(
                root, config, files, sources,
            )
        }
        _ => return None,
    })
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;
