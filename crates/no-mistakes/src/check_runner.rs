pub(crate) mod enabled;
mod fact_collection;
pub(crate) mod finite_set_plan;
pub(crate) mod graph_plan;
pub(crate) mod prepared;
pub(crate) mod results;
mod run_all;

pub(crate) use results::{empty_results, json_value, CheckResults};
pub(crate) use run_all::run_all_with_sql_verdicts;
pub(crate) use run_all::{run_all_with_evidence, run_all_with_options, AnalysisOptions};

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
#[path = "check_runner/coverage_effort/tests.rs"]
mod coverage_effort;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
#[path = "check_runner/download_integrity_effort/tests.rs"]
mod download_integrity_effort;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod low_assignment_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod low_loop_header_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod sql_verdict_annotation_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod sql_verdict_catalog_coverage_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod sql_verdict_conflict_pruning_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod sql_verdict_consumer_recovery_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod sql_verdict_decision_coverage_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod sql_verdict_edge_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod sql_verdict_final_gate_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod sql_verdict_helper_advisory_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod sql_verdict_matrix_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod sql_verdict_modes_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod sql_verdict_obligation_coverage_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod sql_verdict_replay_coverage_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod sql_verdict_sibling_order_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod sql_verdict_subchecks_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod sql_verdict_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
#[path = "check_runner/unique_html_ids_effort/tests.rs"]
mod unique_html_ids_effort;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "check_runner/unique_test_ids_effort/tests.rs"]
mod unique_test_ids_effort;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "check_runner/unmocked_import_effort/tests.rs"]
mod unmocked_import_effort;

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod append_composition_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod append_static_invalid_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod low_reference_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod mutable_registry_low_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod truncated_append_parity_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod unbounded_effort_tests;

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
#[path = "check_runner/payload_compatibility_effort/tests.rs"]
mod payload_compatibility_effort;

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
#[path = "check_runner/finite_set_effort/tests.rs"]
mod finite_set_effort;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "check_runner/transformed_uuid_predicates/tests.rs"]
mod transformed_uuid_predicates;

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod parse_failure_locations_tests;
