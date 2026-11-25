use crate::codebase::rules;
use crate::config::v2::schema::RuleDef;
use anyhow::Result;

#[derive(Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct NoOptions {}

pub(super) fn standard(rule: &RuleDef) -> Result<()> {
    match rule.rule.as_str() {
        rules::agents_md_max_size::RULE_ID => {
            rule.strict_rule_options::<rules::agents_md_max_size::Options>()?;
        }
        rules::ast_grep_ignore_freshness::RULE_ID => {
            rule.strict_rule_options::<rules::ast_grep_ignore_freshness::Options>()?;
        }
        rules::csharp_max_lines_per_file::RULE_ID => {
            rule.strict_rule_options::<rules::csharp_max_lines_per_file::prepared::Options>()?;
        }
        rules::declared_payload_compatibility::RULE_ID => {
            rule.strict_rule_options::<rules::declared_payload_compatibility::DeclaredPayloadCompatibilityOptions>()?;
        }
        rules::DEPENDABOT_COOLDOWN_POLICY => {
            rule.strict_rule_options::<rules::filesystem_dispatch::dependabot_cooldown_policy::Options>()?;
        }
        rules::file_extension_policy::RULE_ID => {
            rule.strict_rule_options::<rules::file_extension_policy::Options>()?;
        }
        rules::forbidden_dependencies::RULE_ID => {
            rule.strict_rule_options::<rules::forbidden_dependencies::options::Options>()?;
        }
        rules::forbidden_workspace_closure::RULE_ID => {
            rule.strict_rule_options::<rules::forbidden_workspace_closure::options::Options>()?;
        }
        rules::lockfile_allowlist::RULE_ID => {
            rule.strict_rule_options::<rules::lockfile_allowlist::Options>()?;
        }
        rules::markdown_eval_tests::RULE_ID => {
            rule.strict_rule_options::<rules::markdown_eval_tests::Options>()?;
        }
        rules::markdown_link_display_text::RULE_ID => {
            rule.strict_rule_options::<rules::markdown_link_display_text::Options>()?;
        }
        rules::markdown_structure_budget::RULE_ID => {
            rule.strict_rule_options::<rules::markdown_structure_budget::Options>()?;
        }
        rules::nextjs_redirect_destinations::RULE_ID => {
            rule.strict_rule_options::<rules::nextjs_redirect_destinations::options::Options>()?;
        }
        rules::no_empty_or_comments_only_files::RULE_ID => {
            rule.strict_rule_options::<rules::no_empty_or_comments_only_files::Options>()?;
        }
        rules::no_git_identity_mutation::RULE_ID => {
            rule.strict_rule_options::<rules::no_git_identity_mutation::Options>()?;
        }
        rules::package_json_export_target_prefix::RULE_ID => {
            rule.strict_rule_options::<rules::package_json_export_target_prefix::Options>()?;
        }
        rules::package_json_nested_workspace_coverage::RULE_ID => {
            rule.strict_rule_options::<rules::package_json_nested_workspace_coverage::Options>()?;
        }
        rules::package_json_registry_only::RULE_ID => {
            rule.strict_rule_options::<rules::package_json_registry_only::Options>()?;
        }
        rules::package_json_required_fields::RULE_ID => {
            rule.strict_rule_options::<rules::package_json_required_fields::Options>()?;
        }
        rules::package_json_workspace_coverage::RULE_ID => {
            rule.strict_rule_options::<rules::package_json_workspace_coverage::Options>()?;
        }
        rules::pnpm_release_age_policy::RULE_ID => {
            rule.strict_rule_options::<rules::pnpm_release_age_policy::Options>()?;
        }
        rules::production_dependency_declarations::RULE_ID => {
            rule.strict_rule_options::<rules::production_dependency_declarations::Options>()?;
        }
        rules::required_companion_imports::RULE_ID => {
            rule.strict_rule_options::<rules::required_companion_imports::Options>()?;
        }
        rules::REQUIRED_DOC_SECTION => {
            rule.strict_rule_options::<rules::required_local_docs::doc_section::DocSectionOptions>(
            )?;
        }
        rules::required_entrypoint_reachability::RULE_ID => {
            rule.strict_rule_options::<rules::required_entrypoint_reachability::Options>()?;
        }
        rules::required_local_docs::RULE_ID => {
            rule.strict_rule_options::<rules::required_local_docs::Options>()?;
        }
        rules::require_files_in_subdirs::RULE_ID => {
            rule.strict_rule_options::<rules::require_files_in_subdirs::Options>()?;
        }
        rules::require_storybook_stories::RULE_ID => {
            rule.strict_rule_options::<rules::require_storybook_stories::types::Options>()?;
        }
        rules::require_test_per_subdir::RULE_ID => {
            rule.strict_rule_options::<rules::require_test_per_subdir::Options>()?;
        }
        rules::RUST_MAX_LINES_PER_FILE => {
            rule.strict_rule_options::<rules::rust_max_lines_per_file::Options>()?;
        }
        rules::RUST_NO_INLINE_ALLOWS => {
            rule.strict_rule_options::<rules::rust_max_lines_per_file::Options>()?;
        }
        rules::RUST_NO_INLINE_TESTS => {
            rule.strict_rule_options::<rules::rust_max_lines_per_file::Options>()?;
        }
        rules::server_route_client_boundary::RULE_ID => {
            rule.strict_rule_options::<rules::server_route_client_boundary::Options>()?;
        }
        rules::shellcheck_runner::RULE_ID => {
            rule.strict_rule_options::<rules::shellcheck_runner::Options>()?;
        }
        rules::strict_package_layout::RULE_ID => {
            rule.strict_rule_options::<rules::strict_package_layout::Options>()?;
        }
        rules::structured_config_policy::RULE_ID => {
            rule.strict_rule_options::<rules::structured_config_policy::Options>()?;
        }
        rules::tsconfig_alias_folder_mapping::RULE_ID => {
            rule.strict_rule_options::<rules::tsconfig_alias_folder_mapping::Options>()?;
        }
        rules::tsconfig_file_coverage::RULE_ID => {
            rule.strict_rule_options::<rules::tsconfig_file_coverage::Options>()?;
        }
        rules::tsconfig_gate_coverage::RULE_ID => {
            rule.strict_rule_options::<rules::tsconfig_gate_coverage::application::Options>()?;
        }
        rules::unconstructed_error_class::RULE_ID => {
            rule.strict_rule_options::<rules::unconstructed_error_class::config::Options>()?;
        }
        rules::vite_optimize_deps_nested_imports::RULE_ID => {
            rule.strict_rule_options::<rules::vite_optimize_deps_nested_imports::Options>()?;
        }
        rules::workspace_package_cycles::RULE_ID => {
            rule.strict_rule_options::<rules::workspace_package_cycles::Options>()?;
        }
        "unique-exports" => {
            rule.strict_rule_options::<crate::codebase::unique_exports::UniqueExportsOptions>()?;
        }
        "route-consistency" => {
            rule.strict_rule_options::<crate::codebase::config::RouteOptions>()?;
        }
        "queue-dashboard-reachability" => {
            rule.strict_rule_options::<crate::codebase::config::QueueOptions>()?;
        }
        "http-route-static-paths" => {
            rule.strict_rule_options::<crate::codebase::config::HttpRouteOptions>()?;
        }
        "http-call-static-paths" => {
            rule.strict_rule_options::<crate::codebase::config::HttpCallOptions>()?;
        }
        _ => {
            rule.strict_rule_options::<NoOptions>()?;
        }
    }
    Ok(())
}
