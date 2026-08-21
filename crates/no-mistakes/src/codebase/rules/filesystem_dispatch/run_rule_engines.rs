use super::run_rule::RunRuleRequest;
use super::*;

pub(super) fn run(request: RunRuleRequest<'_>) -> Result<Vec<RuleFinding>> {
    let RunRuleRequest {
        rule_id,
        fallback,
        root,
        config,
        files,
        sources,
        defer_suppression,
        ..
    } = request;
    match rule_id {
        POSTGRES_CONSTRAINT_VALIDATE => {
            postgres_constraint_validate::check_with_files_and_sources(root, config, files, sources)
        }
        POSTGRES_FK_INDEX => {
            postgres_fk_index::check_with_files_and_sources(root, config, files, sources)
        }
        POSTGRES_REDUNDANT_INDEX => {
            postgres_redundant_index::check_with_files_and_sources(root, config, files, sources)
        }
        POSTGRES_NO_GENERATED_COLUMN_WRITES => {
            postgres_no_generated_column_writes::check_with_files_and_sources(
                root, config, files, sources,
            )
        }
        POSTGRES_LOCK_ORDERING => {
            postgres_lock_ordering::check_with_files_and_sources(root, config, files, sources)
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
        STRUCTURED_CONFIG_POLICY => {
            structured_config_policy::check_with_files_and_sources(root, config, files, sources)
        }
        CONFIG_PATH_REFERENCES => {
            config_path_references::check_with_files_and_sources(root, config, files, sources)
        }
        CSHARP_MAX_LINES_PER_FILE => {
            csharp_max_lines_per_file::check_with_files_sources_and_deferred_suppression(
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
        VERSION_PIN_CONSISTENCY => {
            version_pin_consistency::check_with_files_sources_and_deferred_suppression(
                root,
                config,
                files,
                sources,
                defer_suppression,
            )
        }
        NO_EMPTY_OR_COMMENTS_ONLY_FILES => {
            no_empty_or_comments_only_files::check_with_files_and_sources(
                root, config, files, sources,
            )
        }
        NEXTJS_REDIRECT_DESTINATIONS => {
            nextjs_redirect_destinations::check_with_files_and_sources(root, config, files, sources)
        }
        _ => fallback(root, config, files),
    }
}
