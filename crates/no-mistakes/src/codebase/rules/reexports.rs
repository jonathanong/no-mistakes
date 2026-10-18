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
pub use playwright_project_catalog::{
    prepare_playwright_project_catalog, PreparedPlaywrightProjectCatalog,
};
#[doc(hidden)]
pub use run::{
    canonical_graph_plan, canonical_graph_requires_full_file_universe,
    run_check_with_config_facts_playwright_and_graph,
    run_check_with_config_facts_playwright_and_graph_with_suppression, try_canonical_graph_plan,
};
pub use run::{
    run_check, run_check_with_config_and_facts_and_playwright, run_check_with_facts,
    run_check_with_facts_and_playwright, PreparedRulesCheck,
};
#[doc(hidden)]
pub use vitest_project_catalog::{prepare_vitest_project_catalog, PreparedVitestProjectCatalog};

pub(crate) use file_matching::matching_files;
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

pub(crate) use finding::rule_enabled;
pub use finding::RuleFinding;

mod postgres_findings;
mod sort_findings;
mod variant_findings;
pub(crate) use postgres_findings::{LocatedRuleFinding, PostgresFindings};
pub(crate) use variant_findings::{index_sql_variants, VariantFindingDedup};
mod masked_source;
mod selected_files;
mod target_roots;
pub(crate) use masked_source::{scan_masked_file, scan_masked_files, MaskedFileContext};
mod entrypoints;
pub(crate) use entrypoints::{
    borrowed_source_rule_entrypoints, prepared_file_rule_facts, prepared_rule_entrypoints,
    sourced_file_rule_entrypoints,
};
pub(crate) use selected_files::*;
mod catalog_rule;
mod prepared_facts;
mod prepared_postgres_scan;
mod simple_schema_rule;
mod sql_glob_rules;
pub(crate) use self::{
    catalog_rule::run_catalog_rule, prepared_postgres_scan::collect_prepared_sql_inputs,
    simple_schema_rule::run_simple_schema_rule, sort_findings::sort_findings,
    sql_glob_rules::run_sql_rule_with_glob_filters,
};
pub(crate) use prepared_facts::*;
pub(crate) use rule_context::{PreparedRuleContext, SourcedRuleContext};
pub(crate) use target_roots::{
    file_allowed_by_roots_and_skip, skip_dir_set, target_project_root, target_roots,
    target_roots_with_inferred,
};
