use super::super::{ImpactReason, TestPlan};
use serde::{Deserialize, Serialize};

/// Caller-recorded identity; matching metadata is not cryptographic verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestAuditProvenance {
    pub checkout_revision: String,
    pub source_digest: String,
    pub scope_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestAuditSymbol {
    pub file: String,
    pub symbol: String,
}

/// Wrap an unchanged planner result with the producer's audit scope and identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestAuditPlanArtifact {
    pub schema_version: u32,
    pub provenance: TestAuditProvenance,
    pub plan: TestPlan,
    pub changed_files: Vec<String>,
    /// Refines these files to exact symbol matches instead of any file execution.
    pub changed_symbols: Vec<TestAuditSymbol>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestAuditObservation {
    pub test_file: String,
    pub executed_files: Vec<String>,
    pub executed_symbols: Vec<TestAuditSymbol>,
    /// False retains positive evidence but makes negative evidence unknown.
    pub trace_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestAuditObservationsArtifact {
    pub schema_version: u32,
    pub provenance: TestAuditProvenance,
    pub granularity: String,
    pub suite: String,
    pub complete: bool,
    pub tests: Vec<TestAuditObservation>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestAuditExecutionEvidence {
    pub test_file: String,
    pub matched_files: Vec<String>,
    pub matched_symbols: Vec<TestAuditSymbol>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestAuditSelectionEvidence {
    pub test_file: String,
    pub reasons: Vec<ImpactReason>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestAuditReport {
    pub provenance: TestAuditProvenance,
    pub observed_test_files: usize,
    pub selected_test_files: usize,
    pub selected_observed_tests: Vec<TestAuditExecutionEvidence>,
    pub missed_observed_tests: Vec<TestAuditExecutionEvidence>,
    pub selected_without_observed_execution: Vec<TestAuditSelectionEvidence>,
    pub selected_without_observations: Vec<String>,
    pub selected_with_incomplete_traces: Vec<String>,
    pub limitations: Vec<String>,
}
