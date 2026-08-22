use anyhow::Result;
use std::path::{Path, PathBuf};

use super::{
    agents_md_max_size, banned_paths, banned_renamed_files, config_path_references,
    csharp_max_lines_per_file, doc_consistency, file_extension_policy, finite_set_consistency,
    forbidden_workspace_closure, github_actions_action_timeout_pair,
    github_actions_composite_step_schema, github_actions_job_timeouts, github_actions_pinned_hash,
    github_actions_test_timeout_literals, integration_test_no_mocks, lockfile_allowlist,
    markdown_child_links, markdown_eval_tests, markdown_link_display_text,
    markdown_mermaid_validation, markdown_reachability, markdown_structure_budget,
    nextjs_redirect_destinations, no_empty_or_comments_only_files, no_git_identity_mutation,
    no_mistakes_config, no_raw_ephemeral_port, package_json_registry_only,
    package_json_workspace_coverage, postgres_constraint_validate, postgres_fk_index,
    postgres_lock_ordering, postgres_no_generated_column_writes, postgres_redundant_index,
    production_dependency_declarations, require_files_in_subdirs, require_test_per_subdir,
    required_companion_imports, required_local_docs, rust_rules_combined, shellcheck_runner,
    strict_package_layout, structured_config_policy, test_email_domain_policy,
    test_no_dependency_pins, tsconfig_alias_folder_mapping, tsconfig_file_coverage,
    tsconfig_gate_coverage, version_pin_consistency, vitest_ci_path_coverage,
    vitest_project_mapping, vitest_test_correspondence, workflow_topology_policy,
    workspace_package_cycles,
};

mod candidate_helpers;
mod candidate_index;
mod entrypoints;
mod execute;
mod inventory;
mod markdown_dispatch;
mod metadata;
mod preserved;
mod run_rule;
mod run_rule_engines;
#[macro_use]
mod registry;
use super::{
    rule_enabled, suppress_rule_findings_with_sources_except, RuleFinding, AGENTS_MD_MAX_SIZE,
    BANNED_PATHS, BANNED_RENAMED_FILES, CONFIG_PATH_REFERENCES, CSHARP_MAX_LINES_PER_FILE,
    DOC_CONSISTENCY, FILE_EXTENSION_POLICY, FINITE_SET_CONSISTENCY, FORBIDDEN_WORKSPACE_CLOSURE,
    INTEGRATION_TEST_NO_MOCKS, LOCKFILE_ALLOWLIST, MARKDOWN_CHILD_LINKS, MARKDOWN_EVAL_TESTS,
    MARKDOWN_LINK_DISPLAY_TEXT, MARKDOWN_MERMAID_VALIDATION, MARKDOWN_REACHABILITY,
    MARKDOWN_STRUCTURE_BUDGET, NEXTJS_REDIRECT_DESTINATIONS, NO_EMPTY_OR_COMMENTS_ONLY_FILES,
    NO_GIT_IDENTITY_MUTATION, NO_MISTAKES_CONFIG, NO_RAW_EPHEMERAL_PORT,
    PACKAGE_JSON_REGISTRY_ONLY, PACKAGE_JSON_WORKSPACE_COVERAGE, POSTGRES_CONSTRAINT_VALIDATE,
    POSTGRES_FK_INDEX, POSTGRES_LOCK_ORDERING, POSTGRES_NO_GENERATED_COLUMN_WRITES,
    POSTGRES_REDUNDANT_INDEX, PRODUCTION_DEPENDENCY_DECLARATIONS, REQUIRED_COMPANION_IMPORTS,
    REQUIRED_DOC_SECTION, REQUIRED_LOCAL_DOCS, REQUIRE_FILES_IN_SUBDIRS, REQUIRE_TEST_PER_SUBDIR,
    RUST_MAX_LINES_PER_FILE, RUST_NO_INLINE_ALLOWS, RUST_NO_INLINE_TESTS, SHELLCHECK_RUNNER,
    STRICT_PACKAGE_LAYOUT, STRUCTURED_CONFIG_POLICY, TEST_EMAIL_DOMAIN_POLICY,
    TEST_NO_DEPENDENCY_PINS, TSCONFIG_ALIAS_FOLDER_MAPPING, TSCONFIG_FILE_COVERAGE,
    TSCONFIG_GATE_COVERAGE, VITEST_CI_PATH_COVERAGE, VITEST_PROJECT_MAPPING,
    VITEST_TEST_CORRESPONDENCE, WORKFLOW_TOPOLOGY_POLICY, WORKSPACE_PACKAGE_CYCLES,
};
pub use entrypoints::{
    run_filesystem_rules, run_filesystem_rules_with_config,
    run_filesystem_rules_with_config_and_snapshot,
    run_filesystem_rules_with_config_snapshot_and_vitest_catalog, run_filesystem_rules_with_files,
    run_filesystem_rules_with_visible_and_snapshot,
};
pub(super) const GITHUB_ACTIONS_ACTION_TIMEOUT_PAIR: &str =
    github_actions_action_timeout_pair::RULE_ID;
pub(super) const GITHUB_ACTIONS_COMPOSITE_STEP_SCHEMA: &str =
    github_actions_composite_step_schema::RULE_ID;
pub(super) const GITHUB_ACTIONS_JOB_TIMEOUTS: &str = github_actions_job_timeouts::RULE_ID;
pub(super) const GITHUB_ACTIONS_PINNED_HASH: &str = github_actions_pinned_hash::RULE_ID;
pub(super) const GITHUB_ACTIONS_TEST_TIMEOUT_LITERALS: &str =
    github_actions_test_timeout_literals::RULE_ID;
pub(super) const VERSION_PIN_CONSISTENCY: &str = version_pin_consistency::RULE_ID;

macro_rules! define_filesystem_rule_ids {
    ($($id:expr => $call:path),* $(,)?) => {
        const FILESYSTEM_RULE_IDS: &[&str] = &[
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
        ];
    };
}

crate::filesystem_rules!(define_filesystem_rule_ids);
pub use execute::{
    run_filesystem_rules_with_config_snapshot_catalog_and_sources,
    run_filesystem_rules_with_config_snapshot_catalog_sources_and_facts,
    run_filesystem_rules_with_config_snapshot_catalog_sources_facts_and_suppression,
    PreparedFilesystemRuleInputs,
};

#[cfg(test)]
mod tests;
