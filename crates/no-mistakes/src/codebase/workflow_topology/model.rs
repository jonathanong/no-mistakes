//! Core data model for the GitHub Actions workflow topology graph.
//!
//! This is a faithful port of a standalone TypeScript engine's `types.mts`.
//! The serialized JSON shape produced by [`super::render_json`] is a
//! stability contract (schema v1): **field names, field ORDER, and array
//! sort order must match exactly**. Struct field declaration order below is
//! not arbitrary — it mirrors the TS engine's object-literal construction
//! order (which is what `JSON.stringify` preserves), not the order fields
//! happen to appear in a TS type declaration. Two spots where those two
//! orders diverge are called out inline below; everywhere else they agree.
//!
//! Optional fields use `#[serde(skip_serializing_if = "Option::is_none")]`
//! throughout, matching the TS engine's `...(cond ? {field} : {})` spreads:
//! an absent optional is never emitted, never serialized as `null`.

use super::artifact_types::ArtifactDeclaration;
use super::value_primitives::OrderedJson;
use serde::Serialize;
use std::collections::BTreeMap;

/// Current schema version of the serialized [`WorkflowTopology`] JSON.
pub const WORKFLOW_TOPOLOGY_SCHEMA_VERSION: u32 = 1;

/// A YAML scalar as parsed leniently from workflow YAML (never an object or
/// array). Mirrors the TS engine's `JsonScalar = boolean | number | string`.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(untagged)]
pub enum JsonScalar {
    Bool(bool),
    Number(serde_json::Number),
    Text(String),
}

/// `concurrency.cancel-in-progress` accepts either a literal boolean or an
/// unresolved `${{ ... }}` expression string.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(untagged)]
pub enum ConcurrencyValue {
    Bool(bool),
    Text(String),
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConcurrencyRaw {
    pub group: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_in_progress: Option<ConcurrencyValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConcurrencyEffective {
    pub group: String,
    pub cancel_in_progress: ConcurrencyValue,
    pub queue: String,
}

/// A resolved `concurrency:` block (workflow- or job-level). `raw` preserves
/// exactly what was declared; `effective` fills in GitHub's documented
/// defaults (`cancel-in-progress: false`, `queue: single`).
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct WorkflowConcurrency {
    pub raw: ConcurrencyRaw,
    pub effective: ConcurrencyEffective,
}

/// One entry from a workflow's `on:` block.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct WorkflowTrigger {
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<OrderedJson>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowCallInputType {
    Boolean,
    Number,
    String,
}

impl WorkflowCallInputType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Boolean => "boolean",
            Self::Number => "number",
            Self::String => "string",
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct WorkflowCallInput {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub input_type: Option<WorkflowCallInputType>,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<JsonScalar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct WorkflowCallSecret {
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct WorkflowCallOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A reusable workflow's declared `workflow_call:` contract. Keys are
/// declaration names; `BTreeMap` gives byte-sorted iteration, matching the
/// TS engine's explicit `.toSorted(localeCompare)` before serialization.
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct WorkflowCallContract {
    pub inputs: BTreeMap<String, WorkflowCallInput>,
    pub secrets: BTreeMap<String, WorkflowCallSecret>,
    pub outputs: BTreeMap<String, WorkflowCallOutput>,
}

/// A parsed workflow file.
///
/// Field order (id, path, name, callable, `workflowCall?`, triggers,
/// jobIds, `concurrency?`) matches the TS engine's real object-literal
/// construction order in `parse-workflow.mts`, **not** the order its
/// `WorkflowNodeBase & {callable...}` intersection type declares fields in
/// (which would put `triggers`/`jobIds` before `callable`/`workflowCall`).
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowNode {
    pub id: String,
    pub path: String,
    pub name: String,
    pub callable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_call: Option<WorkflowCallContract>,
    pub triggers: Vec<WorkflowTrigger>,
    pub job_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<WorkflowConcurrency>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StepKind {
    Action,
    Run,
    Other,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowStep {
    pub index: u32,
    pub kind: StepKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uses: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<ArtifactDeclaration>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum JobKind {
    Job,
    MatrixTemplate,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowJobNode {
    pub id: String,
    pub workflow_id: String,
    pub key: String,
    pub kind: JobKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matrix: Option<OrderedJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<WorkflowConcurrency>,
    pub steps: Vec<WorkflowStep>,
}

mod diagnostic_model;
mod edge_model;

pub use diagnostic_model::{DiagnosticCode, Severity, WorkflowTopologyDiagnostic};
pub use edge_model::{
    NeedsEdge, WorkflowCallBindings, WorkflowCallEdge, WorkflowCallSecretsBinding, WorkflowRunEdge,
    WorkflowTopologyEdge,
};

/// The complete workflow topology graph for a repository. This is the
/// top-level type serialized by [`super::render_json`]; see the module
/// docs for the schema-v1 stability contract.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowTopology {
    pub schema_version: u32,
    pub workflows: Vec<WorkflowNode>,
    pub jobs: Vec<WorkflowJobNode>,
    pub edges: Vec<WorkflowTopologyEdge>,
    pub diagnostics: Vec<WorkflowTopologyDiagnostic>,
}
