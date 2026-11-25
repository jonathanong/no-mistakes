use anyhow::Result;
use std::path::{Path, PathBuf};

use super::github_actions_workflow_files::{no_poll_wait_job, pr_diff_range};
use super::{
    agents_md_max_size, ast_grep_ignore_freshness, banned_paths, banned_renamed_files,
    config_path_references, csharp_max_lines_per_file, csharp_no_async_void_delegate,
    csp_allowed_origins, declared_payload_compatibility, doc_consistency, file_extension_policy,
    finite_set_consistency, forbidden_lint_suppressions, forbidden_workspace_closure,
    github_actions_action_input_policy, github_actions_action_timeout_pair,
    github_actions_cache_key_policy, github_actions_cache_setup_policy,
    github_actions_checkout_credentials, github_actions_composite_step_schema,
    github_actions_docker_host_ports, github_actions_download_integrity,
    github_actions_job_permissions, github_actions_job_timeouts,
    github_actions_local_action_checkout, github_actions_max_inline_script_lines,
    github_actions_no_insta_snapshot_updates, github_actions_no_required_pr_path_filters,
    github_actions_pinned_hash, github_actions_script_expressions, github_actions_step_timeouts,
    github_actions_test_timeout_literals, github_actions_vitest_project_selection,
    integration_test_no_mocks, lockfile_allowlist, markdown_child_links, markdown_eval_tests,
    markdown_link_display_text, markdown_mermaid_validation, markdown_reachability,
    markdown_structure_budget, nested_controls, nextjs_redirect_destinations,
    no_empty_or_comments_only_files, no_git_identity_mutation, no_mistakes_config,
    no_raw_ephemeral_port, no_sparse_checkout, no_test_git_sha, package_json_export_target_prefix,
    package_json_nested_workspace_coverage, package_json_registry_only,
    package_json_required_fields, package_json_workspace_coverage, pnpm_overrides_ban,
    pnpm_release_age_policy, postgres_array_columns, postgres_bounded_statements,
    postgres_column_check_constraints, postgres_column_naming, postgres_column_requires_trigger,
    postgres_conflict_ordering, postgres_constraint_validate, postgres_duplicate_function_body,
    postgres_explain_write_rollback, postgres_explicit_columns, postgres_finite_text_columns,
    postgres_fk_index, postgres_generated_column_predicates, postgres_grouping_not_aggregation,
    postgres_idempotent_insert, postgres_identifier_length, postgres_json_relation_ids,
    postgres_key_column_types, postgres_lock_inventory, postgres_lock_ordering,
    postgres_lock_reason, postgres_lock_written_rows_only, postgres_materialized_view_shape,
    postgres_no_add_column, postgres_no_duplicate_keyed_lookups,
    postgres_no_generated_column_writes, postgres_no_offset, postgres_no_test_state_mutation,
    postgres_no_transformed_uuid_predicates, postgres_object_naming, postgres_redundant_index,
    postgres_require_fk_on_delete, postgres_require_named_constraints,
    postgres_require_query_annotation, postgres_required_comments, postgres_required_predicates,
    postgres_sql_shape_policy, postgres_sql_statement_policy,
    postgres_status_with_lifecycle_timestamps, postgres_table_shape,
    production_dependency_declarations, require_files_in_subdirs, require_test_per_subdir,
    required_companion_imports, required_local_docs, rust_rules_combined, shellcheck_runner,
    strict_package_layout, structured_config_policy, swift_no_raw_print,
    swift_no_unguarded_darwin_import, swift_viewmodel_main_actor,
    terraform_iam_no_undocumented_wildcards, test_email_domain_policy, test_fixture_size_cap,
    test_no_dependency_pins, test_reliability, tsconfig_alias_folder_mapping,
    tsconfig_file_coverage, tsconfig_gate_coverage, version_pin_consistency,
    vite_optimize_deps_nested_imports, vitest_ci_path_coverage, vitest_default_excludes,
    vitest_project_mapping, vitest_project_policy, vitest_test_correspondence,
    workflow_topology_policy, workspace_package_cycles,
};

mod call_site_facts;
mod candidate_helpers;
mod candidate_index;
pub(in crate::codebase::rules) mod dependabot_cooldown_policy;
mod entrypoints;
mod execute;
pub mod github_actions_run_command_policy;
pub mod github_actions_workflow_script_paths;
mod inventory;
mod markdown_dispatch;
mod metadata;
mod preserved;
mod run_rule;
mod run_rule_engines;
pub(in crate::codebase::rules) mod shell_compatibility;
#[macro_use]
mod registry;
use super::{
    github_actions_workflow_files, rule_enabled, suppress_rule_findings_with_sources_except,
    RuleFinding, AGENTS_MD_MAX_SIZE, AST_GREP_IGNORE_FRESHNESS, BANNED_PATHS, BANNED_RENAMED_FILES,
    BASH_EMPTY_ARRAY_NOUNSET, CONFIG_PATH_REFERENCES, CSHARP_MAX_LINES_PER_FILE,
    CSHARP_NO_ASYNC_VOID_DELEGATE, CSP_ALLOWED_ORIGINS, DECLARED_PAYLOAD_COMPATIBILITY,
    DEPENDABOT_COOLDOWN_POLICY, DOC_CONSISTENCY, FILE_EXTENSION_POLICY, FINITE_SET_CONSISTENCY,
    FORBIDDEN_LINT_SUPPRESSIONS, FORBIDDEN_WORKSPACE_CLOSURE, INTEGRATION_TEST_NO_MOCKS,
    LOCKFILE_ALLOWLIST, MARKDOWN_CHILD_LINKS, MARKDOWN_EVAL_TESTS, MARKDOWN_LINK_DISPLAY_TEXT,
    MARKDOWN_MERMAID_VALIDATION, MARKDOWN_REACHABILITY, MARKDOWN_STRUCTURE_BUDGET,
    NEXTJS_REDIRECT_DESTINATIONS, NO_EMPTY_OR_COMMENTS_ONLY_FILES, NO_GIT_IDENTITY_MUTATION,
    NO_MISTAKES_CONFIG, NO_RAW_EPHEMERAL_PORT, NO_SPARSE_CHECKOUT, NO_TEST_GIT_SHA,
    PACKAGE_JSON_EXPORT_TARGET_PREFIX, PACKAGE_JSON_NESTED_WORKSPACE_COVERAGE,
    PACKAGE_JSON_REGISTRY_ONLY, PACKAGE_JSON_REQUIRED_FIELDS, PACKAGE_JSON_WORKSPACE_COVERAGE,
    PLAYWRIGHT_NO_SERIAL, PNPM_OVERRIDES_BAN, PNPM_RELEASE_AGE_POLICY, POSTGRES_ARRAY_COLUMNS,
    POSTGRES_BOUNDED_STATEMENTS, POSTGRES_COLUMN_CHECK_CONSTRAINTS, POSTGRES_COLUMN_NAMING,
    POSTGRES_COLUMN_REQUIRES_TRIGGER, POSTGRES_CONFLICT_ORDERING, POSTGRES_CONSTRAINT_VALIDATE,
    POSTGRES_DUPLICATE_FUNCTION_BODY, POSTGRES_EXPLAIN_WRITE_ROLLBACK, POSTGRES_EXPLICIT_COLUMNS,
    POSTGRES_FINITE_TEXT_COLUMNS, POSTGRES_FK_INDEX, POSTGRES_GENERATED_COLUMN_PREDICATES,
    POSTGRES_GROUPING_NOT_AGGREGATION, POSTGRES_IDEMPOTENT_INSERT, POSTGRES_IDENTIFIER_LENGTH,
    POSTGRES_JSON_RELATION_IDS, POSTGRES_KEY_COLUMN_TYPES, POSTGRES_LOCK_INVENTORY,
    POSTGRES_LOCK_ORDERING, POSTGRES_LOCK_REASON, POSTGRES_LOCK_WRITTEN_ROWS_ONLY,
    POSTGRES_MATERIALIZED_VIEW_SHAPE, POSTGRES_NO_ADD_COLUMN, POSTGRES_NO_DUPLICATE_KEYED_LOOKUPS,
    POSTGRES_NO_GENERATED_COLUMN_WRITES, POSTGRES_NO_OFFSET, POSTGRES_NO_TEST_STATE_MUTATION,
    POSTGRES_NO_TRANSFORMED_UUID_PREDICATES, POSTGRES_OBJECT_NAMING, POSTGRES_REDUNDANT_INDEX,
    POSTGRES_REQUIRED_COMMENTS, POSTGRES_REQUIRED_PREDICATES, POSTGRES_REQUIRE_FK_ON_DELETE,
    POSTGRES_REQUIRE_NAMED_CONSTRAINTS, POSTGRES_REQUIRE_QUERY_ANNOTATION,
    POSTGRES_SQL_SHAPE_POLICY, POSTGRES_SQL_STATEMENT_POLICY,
    POSTGRES_STATUS_WITH_LIFECYCLE_TIMESTAMPS, POSTGRES_TABLE_SHAPE,
    PRODUCTION_DEPENDENCY_DECLARATIONS, REACT_NO_NESTED_BUTTONS_AND_LINKS,
    REQUIRED_COMPANION_IMPORTS, REQUIRED_DOC_SECTION, REQUIRED_LOCAL_DOCS,
    REQUIRE_FILES_IN_SUBDIRS, REQUIRE_TEST_PER_SUBDIR, RUST_MAX_LINES_PER_FILE,
    RUST_NO_INLINE_ALLOWS, RUST_NO_INLINE_TESTS, SHELLCHECK_RUNNER, STRICT_PACKAGE_LAYOUT,
    STRUCTURED_CONFIG_POLICY, SWIFT_NO_RAW_PRINT, SWIFT_NO_UNGUARDED_DARWIN_IMPORT,
    SWIFT_VIEWMODEL_MAIN_ACTOR, TERRAFORM_IAM_NO_UNDOCUMENTED_WILDCARDS, TEST_EMAIL_DOMAIN_POLICY,
    TEST_FIXTURE_SIZE_CAP, TEST_NO_DEPENDENCY_PINS, TEST_NO_RETRIES, TSCONFIG_ALIAS_FOLDER_MAPPING,
    TSCONFIG_FILE_COVERAGE, TSCONFIG_GATE_COVERAGE, VITEST_CI_PATH_COVERAGE,
    VITEST_DEFAULT_EXCLUDES, VITEST_PROJECT_MAPPING, VITEST_PROJECT_POLICY,
    VITEST_TEST_CORRESPONDENCE, WORKFLOW_TOPOLOGY_POLICY, WORKSPACE_PACKAGE_CYCLES,
    ZSH_READONLY_PARAMETER,
};
pub use entrypoints::{
    run_filesystem_rules, run_filesystem_rules_with_config,
    run_filesystem_rules_with_config_and_snapshot,
    run_filesystem_rules_with_config_snapshot_and_vitest_catalog, run_filesystem_rules_with_files,
    run_filesystem_rules_with_visible_and_snapshot,
};
pub(super) const GITHUB_ACTIONS_ACTION_INPUT_POLICY: &str =
    github_actions_action_input_policy::RULE_ID;
pub(super) const GITHUB_ACTIONS_ENV_POLICY: &str =
    github_actions_workflow_files::env_policy::RULE_ID;
pub(super) const GITHUB_ACTIONS_ACTION_TIMEOUT_PAIR: &str =
    github_actions_action_timeout_pair::RULE_ID;
pub(super) const GITHUB_ACTIONS_CACHE_KEY_POLICY: &str = github_actions_cache_key_policy::RULE_ID;
pub(super) const GITHUB_ACTIONS_COMPOSITE_STEP_SCHEMA: &str =
    github_actions_composite_step_schema::RULE_ID;
pub(super) const GITHUB_ACTIONS_JOB_TIMEOUTS: &str = github_actions_job_timeouts::RULE_ID;
pub(super) const GITHUB_ACTIONS_LOCAL_ACTION_CHECKOUT: &str =
    github_actions_local_action_checkout::RULE_ID;
pub(super) const GITHUB_ACTIONS_NO_REQUIRED_PR_PATH_FILTERS: &str =
    github_actions_no_required_pr_path_filters::RULE_ID;
pub(super) const GITHUB_ACTIONS_JOB_PERMISSIONS: &str = github_actions_job_permissions::RULE_ID;
pub(super) const GITHUB_ACTIONS_MAX_INLINE_SCRIPT_LINES: &str =
    github_actions_max_inline_script_lines::RULE_ID;
pub(super) const GITHUB_ACTIONS_STEP_TIMEOUTS: &str = github_actions_step_timeouts::RULE_ID;
pub(super) const GITHUB_ACTIONS_NO_INSTA_SNAPSHOT_UPDATES: &str =
    github_actions_no_insta_snapshot_updates::RULE_ID;
pub(super) const GITHUB_ACTIONS_PINNED_HASH: &str = github_actions_pinned_hash::RULE_ID;
pub(super) const GITHUB_ACTIONS_RUN_COMMAND_POLICY: &str =
    github_actions_run_command_policy::RULE_ID;
pub(super) const GITHUB_ACTIONS_WORKFLOW_SCRIPT_PATHS: &str =
    github_actions_workflow_script_paths::RULE_ID;
pub(super) const GITHUB_ACTIONS_SCRIPT_EXPRESSIONS: &str =
    github_actions_script_expressions::RULE_ID;
pub(super) const GITHUB_ACTIONS_CHECKOUT_CREDENTIALS: &str =
    github_actions_checkout_credentials::RULE_ID;
pub(super) const GITHUB_ACTIONS_DOCKER_HOST_PORTS: &str = github_actions_docker_host_ports::RULE_ID;
pub(super) const GITHUB_ACTIONS_NO_POLL_WAIT_JOB: &str = no_poll_wait_job::RULE_ID;
pub(super) const GITHUB_ACTIONS_PR_DIFF_RANGE: &str = pr_diff_range::RULE_ID;
pub(super) const GITHUB_ACTIONS_VITEST_PROJECT_SELECTION: &str =
    github_actions_vitest_project_selection::RULE_ID;
pub(super) const GITHUB_ACTIONS_DOWNLOAD_INTEGRITY: &str =
    github_actions_download_integrity::RULE_ID;
pub(super) const GITHUB_ACTIONS_TEST_TIMEOUT_LITERALS: &str =
    github_actions_test_timeout_literals::RULE_ID;
pub(super) const VERSION_PIN_CONSISTENCY: &str = version_pin_consistency::RULE_ID;
pub(super) const VITE_OPTIMIZE_DEPS_NESTED_IMPORTS: &str =
    vite_optimize_deps_nested_imports::RULE_ID;

macro_rules! define_filesystem_rule_ids {
    ($($id:expr => $call:path => $fixture_options:expr),* $(,)?) => {
        pub(super) const FILESYSTEM_RULE_IDS: &[&str] = &[
            $($id,)*
            MARKDOWN_CHILD_LINKS,
            MARKDOWN_LINK_DISPLAY_TEXT,
            MARKDOWN_MERMAID_VALIDATION,
            MARKDOWN_REACHABILITY,
            MARKDOWN_STRUCTURE_BUDGET,
            RUST_MAX_LINES_PER_FILE,
            RUST_NO_INLINE_TESTS,
            RUST_NO_INLINE_ALLOWS,
            VITEST_PROJECT_MAPPING,
            VITEST_CI_PATH_COVERAGE,
            TSCONFIG_GATE_COVERAGE,
            WORKFLOW_TOPOLOGY_POLICY,
            BASH_EMPTY_ARRAY_NOUNSET,
            ZSH_READONLY_PARAMETER,
            TEST_NO_RETRIES,
            PLAYWRIGHT_NO_SERIAL,
        ];
    };
}

crate::filesystem_rules!(define_filesystem_rule_ids);
pub use execute::{
    run_filesystem_rules_with_config_snapshot_catalog_and_sources,
    run_filesystem_rules_with_config_snapshot_catalog_sources_and_facts,
    run_filesystem_rules_with_config_snapshot_catalog_sources_facts_and_suppression,
    run_filesystem_rules_with_config_snapshot_catalog_sources_facts_suppression_and_session,
    PreparedFilesystemRuleInputs,
};

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;

pub(super) const GITHUB_ACTIONS_CACHE_SETUP_POLICY: &str =
    github_actions_cache_setup_policy::RULE_ID;
