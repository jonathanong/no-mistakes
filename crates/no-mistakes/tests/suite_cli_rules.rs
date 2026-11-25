//! Cli rules integration tests, linked in one harness.

#[path = "cli_diagnostics_fallback.rs"]
mod cli_diagnostics_fallback;

#[path = "cli_check_integration_discovery.rs"]
mod cli_check_integration_discovery;

#[path = "cli_check_partial.rs"]
mod cli_check_partial;
#[path = "cli_check_partial_input_scope.rs"]
mod cli_check_partial_input_scope;
#[path = "cli_check_unused_allow_entries.rs"]
mod cli_check_unused_allow_entries;

#[path = "cli_check_production_exports_class_extends.rs"]
mod cli_check_production_exports_class_extends;
#[path = "cli_check_production_exports_dynamic_selectors.rs"]
mod cli_check_production_exports_dynamic_selectors;
#[path = "cli_check_production_exports_issue_3175_3176.rs"]
mod cli_check_production_exports_issue_3175_3176;
#[path = "cli_check_production_exports_issue_3221.rs"]
mod cli_check_production_exports_issue_3221;
#[path = "cli_check_production_exports_script_entry.rs"]
mod cli_check_production_exports_script_entry;
#[path = "cli_check_production_exports_test_only.rs"]
mod cli_check_production_exports_test_only;

#[path = "cli_check_production_exports_importer_gaps.rs"]
mod cli_check_production_exports_importer_gaps;

#[path = "cli_check_rule_severity.rs"]
mod cli_check_rule_severity;

#[path = "cli_rule_recall_audit.rs"]
mod cli_rule_recall_audit;

#[path = "cli_check_terraform_iam_wildcards.rs"]
mod cli_check_terraform_iam_wildcards;

#[path = "cli_check_cross_package_relative_imports.rs"]
mod cli_check_cross_package_relative_imports;

#[path = "cli_shell_compatibility.rs"]
mod cli_shell_compatibility;

#[path = "cli_shellcheck_policy.rs"]
mod cli_shellcheck_policy;

#[path = "cli_shellcheck_options.rs"]
mod cli_shellcheck_options;

// Share stateless fixture helpers across the modules in this harness.
#[path = "common/check_json_case.rs"]
mod check_json_case;
#[path = "common/check_json_snapshot.rs"]
mod check_json_snapshot;

#[path = "cli_check_react_empty_components.rs"]
mod cli_check_react_empty_components;
#[path = "cli_postgres_snapshot_column_check_constraints.rs"]
mod cli_postgres_snapshot_column_check_constraints;
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
#[cfg(feature = "mermaid-validation")]
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
#[path = "cli_check_rules_test_no_live_repository_scan.rs"]
mod cli_check_rules_test_no_live_repository_scan;
#[path = "cli_check_rules_test_no_live_repository_scan_bounded.rs"]
mod cli_check_rules_test_no_live_repository_scan_bounded;
#[path = "cli_check_rules_test_no_live_repository_scan_caller_roots.rs"]
mod cli_check_rules_test_no_live_repository_scan_caller_roots;
#[path = "cli_check_rules_test_no_live_repository_scan_node_loaders.rs"]
mod cli_check_rules_test_no_live_repository_scan_node_loaders;
#[path = "cli_check_rules_test_no_live_repository_scan_parameter_cause.rs"]
mod cli_check_rules_test_no_live_repository_scan_parameter_cause;
#[path = "cli_check_rules_test_no_live_repository_scan_parameter_roots.rs"]
mod cli_check_rules_test_no_live_repository_scan_parameter_roots;
#[path = "cli_check_rules_test_no_live_repository_scan_provenance.rs"]
mod cli_check_rules_test_no_live_repository_scan_provenance;
#[path = "cli_check_rules_test_no_live_repository_scan_shapes.rs"]
mod cli_check_rules_test_no_live_repository_scan_shapes;
#[path = "cli_check_rules_test_no_live_repository_scan_temporary.rs"]
mod cli_check_rules_test_no_live_repository_scan_temporary;
#[path = "cli_check_rules_tsconfig.rs"]
mod cli_check_rules_tsconfig;
#[path = "cli_check_rules_unconstructed_error_class.rs"]
mod cli_check_rules_unconstructed_error_class;
#[path = "cli_check_rules_version_pin.rs"]
mod cli_check_rules_version_pin;
#[path = "cli_check_rules_workflow_topology.rs"]
mod cli_check_rules_workflow_topology;
#[path = "cli_check_test_reliability.rs"]
mod cli_check_test_reliability;
#[path = "cli_check_unused_disable_directives.rs"]
mod cli_check_unused_disable_directives;
#[path = "cli_gha_snapshot_action_input_policy.rs"]
mod cli_gha_snapshot_action_input_policy;
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
#[path = "cli_gha_snapshot_env_policy.rs"]
mod cli_gha_snapshot_env_policy;
#[path = "cli_gha_snapshot_invalid_yaml.rs"]
mod cli_gha_snapshot_invalid_yaml;
#[path = "cli_gha_snapshot_job_permissions.rs"]
mod cli_gha_snapshot_job_permissions;
#[path = "cli_gha_snapshot_job_timeouts.rs"]
mod cli_gha_snapshot_job_timeouts;
#[path = "cli_gha_snapshot_no_insta_snapshot_updates.rs"]
mod cli_gha_snapshot_no_insta_snapshot_updates;
#[path = "cli_gha_snapshot_no_poll_wait_job.rs"]
mod cli_gha_snapshot_no_poll_wait_job;
#[path = "cli_gha_snapshot_pinned_hash.rs"]
mod cli_gha_snapshot_pinned_hash;
#[path = "cli_gha_snapshot_pr_diff_range.rs"]
mod cli_gha_snapshot_pr_diff_range;
#[path = "cli_gha_snapshot_run_command_policy.rs"]
mod cli_gha_snapshot_run_command_policy;
#[path = "cli_gha_snapshot_script_expressions.rs"]
mod cli_gha_snapshot_script_expressions;
#[path = "cli_gha_snapshot_step_timeouts.rs"]
mod cli_gha_snapshot_step_timeouts;
#[path = "cli_gha_snapshot_test_timeout_literals.rs"]
mod cli_gha_snapshot_test_timeout_literals;
#[path = "cli_gha_snapshot_vitest_project_selection.rs"]
mod cli_gha_snapshot_vitest_project_selection;
#[path = "cli_gha_snapshot_workflow_script_paths.rs"]
mod cli_gha_snapshot_workflow_script_paths;
#[path = "cli_gha_snapshot_workflow_topology.rs"]
mod cli_gha_snapshot_workflow_topology;
#[path = "cli_rra_snapshot_ast_grep_ignore_freshness.rs"]
mod cli_rra_snapshot_ast_grep_ignore_freshness;
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
#[path = "cli_rrb_snapshot_invalid_utf8.rs"]
mod cli_rrb_snapshot_invalid_utf8;
#[path = "cli_rrb_snapshot_nextjs.rs"]
mod cli_rrb_snapshot_nextjs;
#[path = "cli_rrb_snapshot_policy.rs"]
mod cli_rrb_snapshot_policy;
#[path = "cli_rrb_snapshot_scope.rs"]
mod cli_rrb_snapshot_scope;
#[path = "cli_rrb_snapshot_tests.rs"]
mod cli_rrb_snapshot_tests;
#[path = "cli_test_fixture_size_cap.rs"]
mod cli_test_fixture_size_cap;

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

#[path = "cli_gha_snapshot_cache_fd_redirects.rs"]
mod cli_gha_snapshot_cache_fd_redirects;
#[path = "cli_gha_snapshot_cache_key_policy.rs"]
mod cli_gha_snapshot_cache_key_policy;

#[path = "cli_gha_snapshot_cache_setup_policy.rs"]
mod cli_gha_snapshot_cache_setup_policy;

#[path = "cli_gha_snapshot_inline_script_lines.rs"]
mod cli_gha_snapshot_inline_script_lines;

#[path = "cli_gha_snapshot_inline_script_alignment.rs"]
mod cli_gha_snapshot_inline_script_alignment;

#[path = "cli_config_fragments.rs"]
mod cli_config_fragments;

#[path = "cli_config_strict_load.rs"]
mod cli_config_strict_load;

#[path = "cli_rule_options_audit.rs"]
mod cli_rule_options_audit;
#[path = "cli_workflow_option_validation.rs"]
mod cli_workflow_option_validation;

#[path = "cli_gha_snapshot_activity_types.rs"]
mod cli_gha_snapshot_activity_types;

#[path = "cli_gha_snapshot_workflow_policy_bans.rs"]
mod cli_gha_snapshot_workflow_policy_bans;

#[path = "cli_gha_snapshot_local_action_checkout.rs"]
mod cli_gha_snapshot_local_action_checkout;
#[path = "cli_gha_snapshot_required_pr_path_filters.rs"]
mod cli_gha_snapshot_required_pr_path_filters;

#[path = "cli_snapshot_dependabot_cooldown_policy.rs"]
mod cli_snapshot_dependabot_cooldown_policy;

#[path = "cli_snapshot_csp_allowed_origins.rs"]
mod cli_snapshot_csp_allowed_origins;

#[path = "cli_snapshot_vitest_default_excludes.rs"]
mod cli_snapshot_vitest_default_excludes;
#[path = "cli_snapshot_vitest_project_policy.rs"]
mod cli_snapshot_vitest_project_policy;

#[path = "cli_postgres_snapshot_literal_tables.rs"]
mod cli_postgres_snapshot_literal_tables;

#[path = "cli_postgres_snapshot_literal_comment_names.rs"]
mod cli_postgres_snapshot_literal_comment_names;

#[path = "cli_postgres_snapshot_declared_bounds.rs"]
mod cli_postgres_snapshot_declared_bounds;

#[path = "cli_postgres_option_validation.rs"]
mod cli_postgres_option_validation;
#[path = "cli_postgres_rule_timings.rs"]
mod cli_postgres_rule_timings;
#[path = "cli_postgres_snapshot_materialized_view_shape.rs"]
mod cli_postgres_snapshot_materialized_view_shape;
#[path = "cli_postgres_snapshot_no_duplicate_keyed_lookups.rs"]
mod cli_postgres_snapshot_no_duplicate_keyed_lookups;
#[path = "cli_postgres_unused_disables.rs"]
mod cli_postgres_unused_disables;

#[path = "cli_check_rules_test_no_live_repository_scan_conditional_import.rs"]
mod cli_check_rules_test_no_live_repository_scan_conditional_import;
#[path = "cli_check_rules_test_no_live_repository_scan_env.rs"]
mod cli_check_rules_test_no_live_repository_scan_env;
#[path = "cli_check_rules_test_no_live_repository_scan_env_cause.rs"]
mod cli_check_rules_test_no_live_repository_scan_env_cause;

#[path = "cli_check_production_exports_namespaces.rs"]
mod cli_check_production_exports_namespaces;
#[path = "cli_check_production_exports_precision.rs"]
mod cli_check_production_exports_precision;

#[path = "cli_call_site_assembly.rs"]
mod cli_call_site_assembly;
#[path = "cli_check_production_exports_platform.rs"]
mod cli_check_production_exports_platform;

#[path = "cli_check_rules_test_no_live_repository_scan_forwarded.rs"]
mod cli_check_rules_test_no_live_repository_scan_forwarded;

#[path = "cli_check_rules_test_no_live_repository_scan_analyzers.rs"]
mod cli_check_rules_test_no_live_repository_scan_analyzers;
#[path = "cli_check_rules_test_no_live_repository_scan_context.rs"]
mod cli_check_rules_test_no_live_repository_scan_context;
#[path = "cli_check_rules_test_no_live_repository_scan_issue_3286.rs"]
mod cli_check_rules_test_no_live_repository_scan_issue_3286;
#[path = "cli_check_rules_test_no_live_repository_scan_issue_3355.rs"]
mod cli_check_rules_test_no_live_repository_scan_issue_3355;
#[path = "cli_check_rules_test_no_live_repository_scan_test_origins.rs"]
mod cli_check_rules_test_no_live_repository_scan_test_origins;

#[path = "cli_check_rules_test_no_live_repository_scan_issue_3354.rs"]
mod cli_check_rules_test_no_live_repository_scan_issue_3354;

#[path = "cli_check_queue_waits.rs"]
mod cli_check_queue_waits;

#[path = "cli_check_queue_wait_wrappers.rs"]
mod cli_check_queue_wait_wrappers;

#[path = "cli_check_queue_required_options.rs"]
mod cli_check_queue_required_options;

#[path = "cli_check_queue_followups.rs"]
mod cli_check_queue_followups;

#[path = "cli_check_queue_priority.rs"]
mod cli_check_queue_priority;

#[path = "cli_check_queue_diagnostics.rs"]
mod cli_check_queue_diagnostics;

#[path = "cli_check_queue_rate_limit.rs"]
mod cli_check_queue_rate_limit;

#[path = "cli_check_queue_shared_rate_limit.rs"]
mod cli_check_queue_shared_rate_limit;

#[path = "cli_check_rules_no_single_enqueue_in_loop.rs"]
mod cli_check_rules_no_single_enqueue_in_loop;

#[path = "cli_suppression_index.rs"]
mod cli_suppression_index;

#[path = "cli_queue_factory_wrappers.rs"]
mod cli_queue_factory_wrappers;

#[path = "cli_config_rule_validation.rs"]
mod cli_config_rule_validation;

#[path = "cli_react_nested_controls_snapshot.rs"]
mod cli_react_nested_controls_snapshot;

#[cfg(unix)]
#[path = "cli_unreadable_rule_inputs.rs"]
mod cli_unreadable_rule_inputs;
