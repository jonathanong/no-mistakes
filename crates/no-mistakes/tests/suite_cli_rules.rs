//! Cli rules integration tests, linked in one harness.

#[path = "cli_shellcheck_policy.rs"]
mod cli_shellcheck_policy;

// Share stateless fixture helpers across the modules in this harness.
#[path = "common/git_tracked_rule.rs"]
mod git_tracked_rule;
#[path = "common/gitignore_fixture.rs"]
mod gitignore_fixture;
#[path = "common/saved_fixture.rs"]
mod saved_fixture;

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

#[path = "cli_check_git_identity_prefixes.rs"]
mod cli_check_git_identity_prefixes;

#[path = "cli_check_storybook_effort.rs"]
mod cli_check_storybook_effort;

#[path = "cli_check_version_pin_policy.rs"]
mod cli_check_version_pin_policy;
#[path = "cli_check_workflow_unreadable_policy.rs"]
mod cli_check_workflow_unreadable_policy;

#[path = "cli_coverage_assertion_owners.rs"]
mod cli_coverage_assertion_owners;
