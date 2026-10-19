pub(crate) mod enabled;
mod fact_collection;
pub(crate) mod finite_set_plan;
mod graph_plan;
pub(crate) mod prepared;
pub(crate) mod results;
mod run_all;

pub(crate) use results::{complete_domain_checks, empty_results, json_value, CheckResults};
pub(crate) use run_all::run_all_with_evidence;
pub(crate) use run_all::run_all_with_sql_verdicts;

#[cfg(test)]
mod sql_verdict_annotation_tests;
#[cfg(test)]
mod sql_verdict_catalog_coverage_tests;
#[cfg(test)]
mod sql_verdict_consumer_recovery_tests;
#[cfg(test)]
mod sql_verdict_decision_coverage_tests;
#[cfg(test)]
mod sql_verdict_edge_tests;
#[cfg(test)]
mod sql_verdict_final_gate_tests;
#[cfg(test)]
mod sql_verdict_matrix_tests;
#[cfg(test)]
mod sql_verdict_modes_tests;
#[cfg(test)]
mod sql_verdict_obligation_coverage_tests;
#[cfg(test)]
mod sql_verdict_replay_coverage_tests;
#[cfg(test)]
mod sql_verdict_subchecks_tests;
#[cfg(test)]
mod sql_verdict_tests;
#[cfg(test)]
mod tests;
