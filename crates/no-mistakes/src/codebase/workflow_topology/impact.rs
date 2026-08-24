//! Revision-aware impact projection over the stable workflow-topology graph.
//!
//! The topology schema remains version 1.  This module intentionally exposes
//! a separate, versioned report because callers need base/head provenance and
//! fail-open diagnostics that do not belong in a snapshot graph.

mod actions;
mod diagnostics;
mod job_selection;
mod project;
mod reachability;
mod repository;
mod snapshot;
mod workflow_graph;
mod yaml;

pub(crate) use repository::topology_impact_report;

use serde::Serialize;

pub const CI_TOPOLOGY_IMPACT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CiTopologyImpactDiagnostic {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_path: Option<String>,
    pub scope: CiTopologyImpactDiagnosticScope,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_job_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CiTopologyImpactDiagnosticScope {
    Localized,
    Global,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CiTopologyImpactReport {
    pub schema_version: u32,
    pub base_revision: String,
    pub head_revision: String,
    pub changed_paths: Vec<String>,
    pub affected_workflows: Vec<String>,
    pub affected_root_job_ids: Vec<String>,
    pub diagnostics: Vec<CiTopologyImpactDiagnostic>,
    pub global_fallback: bool,
}

#[cfg(test)]
#[path = "impact_tests_regressions.rs"]
mod regression_tests;
#[cfg(test)]
#[path = "impact_tests.rs"]
mod tests;
