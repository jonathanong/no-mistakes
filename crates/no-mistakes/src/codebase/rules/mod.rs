pub mod agents_md_max_size;
pub mod banned_paths;
pub mod banned_renamed_files;
pub mod config_path_references;
pub mod csharp_max_lines_per_file;
pub mod doc_consistency;
pub mod file_extension_policy;
mod file_matching;
pub mod finite_set_consistency;
pub mod forbidden_dependencies;
pub mod forbidden_workspace_closure;
pub mod github_actions_composite_step_schema;
pub mod github_actions_job_timeouts;
pub mod github_actions_pinned_hash;
mod ids;
pub mod integration_test_no_mocks;
pub mod lockfile_allowlist;
pub mod markdown_child_links;
pub mod markdown_eval_tests;
pub(crate) mod markdown_facts;
pub mod markdown_link_display_text;
pub mod markdown_mermaid_validation;
pub mod markdown_reachability;
pub(crate) mod markdown_scope;
pub mod markdown_structure_budget;
pub mod nextjs_no_api_routes;
pub mod nextjs_no_caching;
pub mod nextjs_redirect_destinations;
pub mod no_empty_or_comments_only_files;
pub mod no_git_identity_mutation;
pub mod package_json_registry_only;
pub mod package_json_workspace_coverage;
pub mod postgres_constraint_validate;
pub mod postgres_fk_index;
pub mod postgres_lock_ordering;
pub mod postgres_no_generated_column_writes;
pub mod production_dependency_declarations;
pub mod require_files_in_subdirs;
pub mod require_storybook_stories;
pub mod require_test_per_subdir;
pub mod required_companion_imports;
pub mod required_entrypoint_reachability;
pub mod required_local_docs;
mod roots;
pub mod rust_max_lines_per_file;
pub mod rust_no_inline_allows;
pub mod rust_no_inline_tests;
pub mod rust_rules_combined;
pub mod server_route_client_boundary;
pub mod shellcheck_runner;
pub mod strict_package_layout;
pub mod structured_config_policy;
pub mod test_email_domain_policy;
pub mod test_no_dependency_pins;
pub mod test_no_unmocked_dynamic_imports;
pub mod tsconfig_alias_folder_mapping;
pub mod tsconfig_file_coverage;
pub mod tsconfig_gate_coverage;
pub mod vitest_ci_path_coverage;
mod vitest_project_catalog;
pub mod vitest_project_mapping;
pub mod vitest_test_correspondence;
pub mod workspace_package_cycles;

pub mod filesystem_dispatch;
pub(crate) mod path_filter;
mod run;
mod source_access;
mod suppression;

use serde::Serialize;

pub use filesystem_dispatch::{
    run_filesystem_rules, run_filesystem_rules_with_config,
    run_filesystem_rules_with_config_and_snapshot,
    run_filesystem_rules_with_config_snapshot_and_vitest_catalog,
    run_filesystem_rules_with_config_snapshot_catalog_and_sources,
    run_filesystem_rules_with_config_snapshot_catalog_sources_and_facts,
    run_filesystem_rules_with_config_snapshot_catalog_sources_facts_and_suppression,
    run_filesystem_rules_with_files, run_filesystem_rules_with_visible_and_snapshot,
};
pub use ids::*;
#[doc(hidden)]
pub use run::run_check_with_config_facts_playwright_and_graph_with_suppression;
#[doc(hidden)]
pub use run::{
    canonical_graph_plan, canonical_graph_requires_full_file_universe,
    run_check_with_config_facts_playwright_and_graph,
};
pub use run::{
    run_check, run_check_with_config_and_facts_and_playwright, run_check_with_facts,
    run_check_with_facts_and_playwright, PreparedRulesCheck,
};
#[doc(hidden)]
pub use vitest_project_catalog::{prepare_vitest_project_catalog, PreparedVitestProjectCatalog};

pub(crate) use file_matching::matching_files;
pub(crate) use roots::{
    file_allowed_by_roots_and_skip, rule_enabled, skip_dir_set, target_project_root, target_roots,
    target_roots_with_inferred,
};
pub(crate) use source_access::{read_source, source_store_for_files};
#[doc(hidden)]
pub use suppression::{
    suppress_domain_findings_with_source_files, suppress_domain_findings_with_source_locations,
    suppress_domain_findings_with_sources, SuppressedFinding, SuppressionTarget,
};
pub(crate) use suppression::{
    suppress_rule_findings_with_source, suppress_rule_findings_with_sources,
    suppress_rule_findings_with_sources_except,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleFinding {
    pub rule: String,
    pub file: String,
    pub line: usize,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

mod sort_findings;
pub(crate) use sort_findings::sort_findings;

#[cfg(test)]
mod suppression_absolute_paths_tests;
#[cfg(test)]
mod suppression_tests;
#[cfg(test)]
mod target_roots_tests;
#[cfg(test)]
mod tests;
