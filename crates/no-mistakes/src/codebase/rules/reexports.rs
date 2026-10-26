pub use super::filesystem_dispatch::{
    run_filesystem_rules, run_filesystem_rules_with_config,
    run_filesystem_rules_with_config_and_snapshot,
    run_filesystem_rules_with_config_snapshot_and_vitest_catalog,
    run_filesystem_rules_with_config_snapshot_catalog_and_sources,
    run_filesystem_rules_with_config_snapshot_catalog_sources_and_facts,
    run_filesystem_rules_with_config_snapshot_catalog_sources_facts_and_suppression,
    run_filesystem_rules_with_files, run_filesystem_rules_with_visible_and_snapshot,
};
pub use super::ids::*;
#[doc(hidden)]
pub use super::playwright_project_catalog::{
    prepare_playwright_project_catalog, PreparedPlaywrightProjectCatalog,
};
#[doc(hidden)]
pub use super::run::{
    canonical_graph_plan, canonical_graph_requires_full_file_universe,
    run_check_with_config_facts_playwright_and_graph,
    run_check_with_config_facts_playwright_and_graph_with_suppression, try_canonical_graph_plan,
};
pub use super::run::{
    run_check, run_check_with_config_and_facts_and_playwright, run_check_with_facts,
    run_check_with_facts_and_playwright, PreparedRulesCheck,
};
#[doc(hidden)]
pub use super::vitest_project_catalog::{
    prepare_vitest_project_catalog, PreparedVitestProjectCatalog,
};

pub(crate) use super::file_matching::matching_files;
pub(crate) use super::selected_files::selected_rule_files;
pub(crate) use super::source_access::{read_source, source_store_for_files};
#[doc(hidden)]
pub use super::suppression::{
    suppress_domain_findings_with_source_files, suppress_domain_findings_with_source_locations,
    suppress_domain_findings_with_sources, SuppressedFinding, SuppressionTarget,
};
pub(crate) use super::suppression::{
    suppress_rule_findings_with_source, suppress_rule_findings_with_sources,
    suppress_rule_findings_with_sources_except,
};

pub(crate) use super::finding::rule_enabled;
pub use super::finding::RuleFinding;

pub(crate) use super::entrypoints::{
    borrowed_source_rule_entrypoints, diagnostic_source_rule_entrypoints, prepared_file_rule_facts,
    prepared_rule_entrypoints, sourced_file_rule_entrypoints,
};
pub(crate) use super::masked_source::{scan_masked_file, scan_masked_files, MaskedFileContext};
pub(crate) use super::postgres_findings::{LocatedRuleFinding, PostgresFindings};
pub(crate) use super::prepared_facts::*;
pub(crate) use super::rule_context::{PreparedRuleContext, SourcedRuleContext};
pub(crate) use super::selected_files::*;
pub(crate) use super::target_roots::{
    file_allowed_by_roots_and_skip, skip_dir_set, target_project_root, target_roots,
    target_roots_with_inferred,
};
pub(crate) use super::variant_findings::{index_sql_variants, VariantFindingDedup};
pub(crate) use super::{
    catalog_rule::run_catalog_rule, prepared_postgres_scan::collect_prepared_sql_inputs,
    simple_schema_rule::run_simple_schema_rule, sort_findings::sort_findings,
    sql_glob_rules::run_sql_rule_with_glob_filters,
};
