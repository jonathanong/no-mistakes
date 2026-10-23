pub(crate) mod enabled;
mod fact_collection;
pub(crate) mod finite_set_plan;
mod graph_plan;
pub(crate) mod prepared;
pub(crate) mod results;
mod run_all;

pub(crate) use results::{complete_domain_checks, empty_results, json_value, CheckResults};
pub(crate) use run_all::run_all_with_sql_verdicts;
pub(crate) use run_all::{run_all_with_evidence, run_all_with_options, AnalysisOptions};

#[cfg(test)]
#[path = "check_runner/coverage_effort/tests.rs"]
mod coverage_effort;
#[cfg(test)]
mod low_assignment_tests;
#[cfg(test)]
mod low_loop_header_tests;
#[cfg(test)]
mod sql_verdict_annotation_tests;
#[cfg(test)]
mod sql_verdict_catalog_coverage_tests;
#[cfg(test)]
mod sql_verdict_conflict_pruning_tests;
#[cfg(test)]
mod sql_verdict_consumer_recovery_tests;
#[cfg(test)]
mod sql_verdict_decision_coverage_tests;
#[cfg(test)]
mod sql_verdict_edge_tests;
#[cfg(test)]
mod sql_verdict_final_gate_tests;
#[cfg(test)]
mod sql_verdict_helper_advisory_tests;
#[cfg(test)]
mod sql_verdict_matrix_tests;
#[cfg(test)]
mod sql_verdict_modes_tests;
#[cfg(test)]
mod sql_verdict_obligation_coverage_tests;
#[cfg(test)]
mod sql_verdict_replay_coverage_tests;
#[cfg(test)]
mod sql_verdict_sibling_order_tests;
#[cfg(test)]
mod sql_verdict_subchecks_tests;
#[cfg(test)]
mod sql_verdict_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
#[path = "check_runner/unique_test_ids_effort/tests.rs"]
mod unique_test_ids_effort;
#[cfg(test)]
#[path = "check_runner/unmocked_import_effort/tests.rs"]
mod unmocked_import_effort;

#[cfg(test)]
mod append_composition_tests;
#[cfg(test)]
mod append_static_invalid_tests;
#[cfg(test)]
mod low_reference_tests;
#[cfg(test)]
mod truncated_append_parity_tests;
#[cfg(test)]
mod mutable_registry_low_tests;
#[cfg(test)]
mod unbounded_effort_tests;
