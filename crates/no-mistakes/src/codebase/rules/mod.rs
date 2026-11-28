pub mod advisory_rules;
pub(crate) mod allow_audit;
mod byte_prefilter;
pub(crate) mod call_policy;
mod catalog_rule;
mod config_diagnostic;
mod configured_root;
#[doc(hidden)]
pub mod disable_directive_audit;
mod entrypoints;
#[doc(hidden)]
pub mod file_diagnostics;
mod file_matching;
pub mod filesystem_dispatch;
mod finding;
mod graph_source_prerequisites;
mod ids;
mod lexical_mask;
mod manifest_analysis;
mod masked_source;
mod native_sql_advice;
pub(crate) mod path_filter;
mod playwright_project_catalog;
mod prepared_facts;
mod prepared_postgres_scan;
mod reexports;
mod rule_context;
#[doc(hidden)]
pub mod rule_ids;
mod run;
mod runner_project_catalog;
mod selected_files;
mod simple_schema_rule;
mod sort_findings;
mod source_access;
mod sql_glob_rules;
mod suppression;
mod target_roots;
mod validate_options;
mod variant_findings;
mod vitest_project_catalog;

include!("mod_docs.rs");
include!("mod_github_actions.rs");
include!("mod_languages_and_code.rs");
include!("mod_packages_and_config.rs");
include!("mod_postgres.rs");
include!("mod_testing.rs");

pub use reexports::*;

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod postgres_executor_options_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
#[path = "prepared_facts/tests.rs"]
mod prepared_facts_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod query_reached_per_item_prepared_tests;
#[cfg(test)]
mod sql_rule_observer_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod suppression_absolute_paths_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod suppression_source_filter_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod suppression_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod target_roots_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
