//! Cli rules integration tests, linked in one harness.

#[path = "cli_check_production_exports_test_only.rs"]
mod cli_check_production_exports_test_only;

#[path = "cli_shellcheck_policy.rs"]
mod cli_shellcheck_policy;

// Share stateless fixture helpers across the modules in this harness.
#[path = "common/check_json_snapshot.rs"]
mod check_json_snapshot;
#[path = "common/git_tracked_rule.rs"]
mod git_tracked_rule;
#[path = "common/gitignore_fixture.rs"]
mod gitignore_fixture;
#[path = "common/policy_advisories.rs"]
mod policy_advisories;
#[path = "common/saved_fixture.rs"]
mod saved_fixture;

#[path = "cli_checkout_credentials.rs"]
mod cli_checkout_credentials;

#[path = "cli_check_banned_paths.rs"]
mod cli_check_banned_paths;
#[path = "cli_check_deadline_evidence.rs"]
mod cli_check_deadline_evidence;
#[path = "cli_check_declared_payload.rs"]
mod cli_check_declared_payload;
#[path = "cli_check_finite_set.rs"]
mod cli_check_finite_set;
#[path = "cli_check_mermaid.rs"]
mod cli_check_mermaid;
#[path = "cli_check_redirect_delegation.rs"]
mod cli_check_redirect_delegation;
#[path = "cli_check_repository_inventory.rs"]
mod cli_check_repository_inventory;
#[path = "cli_check_rules.rs"]
mod cli_check_rules;
#[path = "cli_check_rules2.rs"]
mod cli_check_rules2;
#[path = "cli_check_rules_agents.rs"]
mod cli_check_rules_agents;
#[path = "cli_check_rules_config_path_references.rs"]
mod cli_check_rules_config_path_references;
#[path = "cli_check_rules_content.rs"]
mod cli_check_rules_content;
#[path = "cli_check_rules_csharp.rs"]
mod cli_check_rules_csharp;
#[path = "cli_check_rules_ephemeral.rs"]
mod cli_check_rules_ephemeral;
#[path = "cli_check_rules_forbidden_calls.rs"]
mod cli_check_rules_forbidden_calls;
#[path = "cli_check_rules_forbidden_calls_arguments.rs"]
mod cli_check_rules_forbidden_calls_arguments;
#[path = "cli_check_rules_forbidden_calls_contract.rs"]
mod cli_check_rules_forbidden_calls_contract;
#[path = "cli_check_rules_forbidden_calls_extends.rs"]
mod cli_check_rules_forbidden_calls_extends;
#[path = "cli_check_rules_forbidden_calls_matrix.rs"]
mod cli_check_rules_forbidden_calls_matrix;
#[path = "cli_check_rules_forbidden_calls_workspace_package.rs"]
mod cli_check_rules_forbidden_calls_workspace_package;
#[path = "cli_check_rules_github.rs"]
mod cli_check_rules_github;
#[path = "cli_check_rules_gitignored.rs"]
mod cli_check_rules_gitignored;
#[path = "cli_check_rules_lexical_masking.rs"]
mod cli_check_rules_lexical_masking;
#[path = "cli_check_rules_markdown_policy.rs"]
mod cli_check_rules_markdown_policy;
#[path = "cli_check_rules_nextjs.rs"]
mod cli_check_rules_nextjs;
#[path = "cli_check_rules_nextjs_tracked.rs"]
mod cli_check_rules_nextjs_tracked;
#[path = "cli_check_rules_package_json_nested_workspace_coverage.rs"]
mod cli_check_rules_package_json_nested_workspace_coverage;
#[path = "cli_check_rules_package_json_required_fields.rs"]
mod cli_check_rules_package_json_required_fields;
#[path = "cli_check_rules_pnpm_overrides_ban.rs"]
mod cli_check_rules_pnpm_overrides_ban;
#[path = "cli_check_rules_pnpm_release_age_policy.rs"]
mod cli_check_rules_pnpm_release_age_policy;
#[path = "cli_check_rules_repository_guards.rs"]
mod cli_check_rules_repository_guards;
#[path = "cli_check_rules_swift.rs"]
mod cli_check_rules_swift;
#[path = "cli_check_rules_tsconfig.rs"]
mod cli_check_rules_tsconfig;
#[path = "cli_check_rules_unconstructed_error_class.rs"]
mod cli_check_rules_unconstructed_error_class;
#[path = "cli_check_rules_version_pin.rs"]
mod cli_check_rules_version_pin;
#[path = "cli_check_rules_workflow_topology.rs"]
mod cli_check_rules_workflow_topology;
#[path = "cli_check_unused_disable_directives.rs"]
mod cli_check_unused_disable_directives;
#[path = "cli_gha_snapshot_action_timeout_pair.rs"]
mod cli_gha_snapshot_action_timeout_pair;
#[path = "cli_gha_snapshot_checkout_credentials.rs"]
mod cli_gha_snapshot_checkout_credentials;
#[path = "cli_gha_snapshot_composite_step_schema.rs"]
mod cli_gha_snapshot_composite_step_schema;
#[path = "cli_gha_snapshot_docker_host_ports.rs"]
mod cli_gha_snapshot_docker_host_ports;
#[path = "cli_gha_snapshot_download_integrity.rs"]
mod cli_gha_snapshot_download_integrity;
#[path = "cli_gha_snapshot_job_timeouts.rs"]
mod cli_gha_snapshot_job_timeouts;
#[path = "cli_gha_snapshot_pinned_hash.rs"]
mod cli_gha_snapshot_pinned_hash;
#[path = "cli_gha_snapshot_script_expressions.rs"]
mod cli_gha_snapshot_script_expressions;
#[path = "cli_gha_snapshot_step_timeouts.rs"]
mod cli_gha_snapshot_step_timeouts;
#[path = "cli_gha_snapshot_test_timeout_literals.rs"]
mod cli_gha_snapshot_test_timeout_literals;
#[path = "cli_gha_snapshot_workflow_topology.rs"]
mod cli_gha_snapshot_workflow_topology;
#[path = "cli_rra_snapshot_banned_paths.rs"]
mod cli_rra_snapshot_banned_paths;
#[path = "cli_rra_snapshot_banned_renamed_files.rs"]
mod cli_rra_snapshot_banned_renamed_files;
#[path = "cli_rra_snapshot_no_empty_or_comments_only_files.rs"]
mod cli_rra_snapshot_no_empty_or_comments_only_files;
#[path = "cli_rra_snapshot_no_git_identity_mutation.rs"]
mod cli_rra_snapshot_no_git_identity_mutation;
#[path = "cli_rra_snapshot_no_mistakes_config.rs"]
mod cli_rra_snapshot_no_mistakes_config;
#[path = "cli_rra_snapshot_no_raw_ephemeral_port.rs"]
mod cli_rra_snapshot_no_raw_ephemeral_port;
#[path = "cli_rra_snapshot_no_sparse_checkout.rs"]
mod cli_rra_snapshot_no_sparse_checkout;
#[path = "cli_rra_snapshot_no_test_git_sha.rs"]
mod cli_rra_snapshot_no_test_git_sha;
#[path = "cli_rra_snapshot_require_files_in_subdirs.rs"]
mod cli_rra_snapshot_require_files_in_subdirs;
#[path = "cli_rra_snapshot_require_storybook_stories.rs"]
mod cli_rra_snapshot_require_storybook_stories;
#[path = "cli_rra_snapshot_require_test_per_subdir.rs"]
mod cli_rra_snapshot_require_test_per_subdir;
#[path = "cli_rra_snapshot_strict_package_layout.rs"]
mod cli_rra_snapshot_strict_package_layout;
#[path = "cli_rra_snapshot_structured_config_policy.rs"]
mod cli_rra_snapshot_structured_config_policy;
#[path = "cli_rra_snapshot_unconstructed_error_class.rs"]
mod cli_rra_snapshot_unconstructed_error_class;

#[path = "cli_check_git_identity_prefixes.rs"]
mod cli_check_git_identity_prefixes;

#[path = "cli_check_storybook_effort.rs"]
mod cli_check_storybook_effort;

#[path = "cli_check_version_pin_policy.rs"]
mod cli_check_version_pin_policy;
#[path = "cli_check_workflow_unreadable_policy.rs"]
mod cli_check_workflow_unreadable_policy;

#[path = "cli_check_file_input_analysis.rs"]
mod cli_check_file_input_analysis;

#[path = "cli_check_constructor_resolution.rs"]
mod cli_check_constructor_resolution;

#[path = "cli_check_playwright_patterns.rs"]
mod cli_check_playwright_patterns;

#[path = "cli_check_playwright_runner_warnings.rs"]
mod cli_check_playwright_runner_warnings;

#[path = "cli_manifest_unanalyzable.rs"]
mod cli_manifest_unanalyzable;

#[path = "cli_coverage_assertion_owners.rs"]
mod cli_coverage_assertion_owners;
