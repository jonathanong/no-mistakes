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

use super::artifact_types::{ArtifactDeclaration, ArtifactEdge};
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

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct NeedsEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum WorkflowCallSecretsBinding {
    Inherit,
    Explicit {
        values: BTreeMap<String, JsonScalar>,
    },
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct WorkflowCallBindings {
    pub inputs: BTreeMap<String, JsonScalar>,
    pub secrets: WorkflowCallSecretsBinding,
}

/// A `uses:` reusable-workflow call from a job. Field order is
/// `from, target, local, bindings, to?` — **`to` is appended last** at the
/// TS engine's real construction site (`callEdge` in `workflow-values.mts`),
/// not right after `from` as `WorkflowCallEdge`'s type declaration order
/// would suggest. `to` is present only for local (`./`) calls.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowCallEdge {
    pub from: String,
    pub target: String,
    pub local: bool,
    pub bindings: WorkflowCallBindings,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowRunEdge {
    pub from: String,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branches: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branches_ignore: Option<Vec<String>>,
}

/// The four typed edge kinds in the topology graph. `#[serde(tag = "kind")]`
/// always serializes the tag first regardless of the wrapped struct's own
/// field order, which matches every edge construction site in the TS engine
/// (`kind` is always the first key of the object literal).
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum WorkflowTopologyEdge {
    Needs(NeedsEdge),
    Calls(WorkflowCallEdge),
    WorkflowRun(WorkflowRunEdge),
    Artifact(ArtifactEdge),
}

impl WorkflowTopologyEdge {
    /// The workflow (or job's workflow) this edge originates from.
    pub fn from(&self) -> &str {
        match self {
            Self::Needs(edge) => &edge.from,
            Self::Calls(edge) => &edge.from,
            Self::WorkflowRun(edge) => &edge.from,
            Self::Artifact(edge) => &edge.from,
        }
    }

    /// The edge's destination, when it has one. Remote (`calls`, non-local)
    /// edges have no resolvable destination.
    pub fn to(&self) -> Option<&str> {
        match self {
            Self::Needs(edge) => Some(&edge.to),
            Self::Calls(edge) => edge.to.as_deref(),
            Self::WorkflowRun(edge) => Some(&edge.to),
            Self::Artifact(edge) => Some(&edge.to),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticCode {
    MalformedWorkflow,
    MissingNeedsDependency,
    JobDependencyCycle,
    DuplicateStepId,
    UnknownStepReference,
    NonPriorStepReference,
    DuplicateWorkflowName,
    MissingLocalWorkflow,
    NonCallableWorkflow,
    WorkflowCallCycle,
    MissingWorkflowRunSource,
    AmbiguousWorkflowRunSource,
    WorkflowRunCycle,
    WorkflowRunChainLimit,
    UnknownWorkflowFilter,
    InvalidWorkflowFilter,
    MissingWorkflowCallInput,
    UnknownWorkflowCallInput,
    WorkflowCallInputTypeMismatch,
    MissingWorkflowCallSecret,
    UnknownWorkflowCallSecret,
    UnknownWorkflowCallOutput,
    MissingArtifactProducer,
    AmbiguousArtifactProducer,
    ArtifactResolutionLimit,
}

impl DiagnosticCode {
    /// The kebab-case wire value (matches the `#[serde(rename_all =
    /// "kebab-case")]` serialization) — used for sort-key computation
    /// without round-tripping through JSON.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MalformedWorkflow => "malformed-workflow",
            Self::MissingNeedsDependency => "missing-needs-dependency",
            Self::JobDependencyCycle => "job-dependency-cycle",
            Self::DuplicateStepId => "duplicate-step-id",
            Self::UnknownStepReference => "unknown-step-reference",
            Self::NonPriorStepReference => "non-prior-step-reference",
            Self::DuplicateWorkflowName => "duplicate-workflow-name",
            Self::MissingLocalWorkflow => "missing-local-workflow",
            Self::NonCallableWorkflow => "non-callable-workflow",
            Self::WorkflowCallCycle => "workflow-call-cycle",
            Self::MissingWorkflowRunSource => "missing-workflow-run-source",
            Self::AmbiguousWorkflowRunSource => "ambiguous-workflow-run-source",
            Self::WorkflowRunCycle => "workflow-run-cycle",
            Self::WorkflowRunChainLimit => "workflow-run-chain-limit",
            Self::UnknownWorkflowFilter => "unknown-workflow-filter",
            Self::InvalidWorkflowFilter => "invalid-workflow-filter",
            Self::MissingWorkflowCallInput => "missing-workflow-call-input",
            Self::UnknownWorkflowCallInput => "unknown-workflow-call-input",
            Self::WorkflowCallInputTypeMismatch => "workflow-call-input-type-mismatch",
            Self::MissingWorkflowCallSecret => "missing-workflow-call-secret",
            Self::UnknownWorkflowCallSecret => "unknown-workflow-call-secret",
            Self::UnknownWorkflowCallOutput => "unknown-workflow-call-output",
            Self::MissingArtifactProducer => "missing-artifact-producer",
            Self::AmbiguousArtifactProducer => "ambiguous-artifact-producer",
            Self::ArtifactResolutionLimit => "artifact-resolution-limit",
        }
    }
}

/// Field order (severity, code, message, workflowPath, jobId?, stepIndex?,
/// callJobId?, calleeWorkflowPath?) matches every diagnostic construction
/// site in the TS engine, which happens to also match its type declaration.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowTopologyDiagnostic {
    pub severity: Severity,
    pub code: DiagnosticCode,
    pub message: String,
    pub workflow_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_job_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callee_workflow_path: Option<String>,
}

impl WorkflowTopologyDiagnostic {
    /// A minimal error diagnostic; chain the `with_*` builders below for
    /// the optional fields a specific diagnostic code needs.
    pub fn new(
        code: DiagnosticCode,
        message: impl Into<String>,
        workflow_path: impl Into<String>,
    ) -> Self {
        Self {
            severity: Severity::Error,
            code,
            message: message.into(),
            workflow_path: workflow_path.into(),
            job_id: None,
            step_index: None,
            call_job_id: None,
            callee_workflow_path: None,
        }
    }

    pub fn with_job(mut self, job_id: impl Into<String>) -> Self {
        self.job_id = Some(job_id.into());
        self
    }

    pub fn with_step(mut self, step_index: u32) -> Self {
        self.step_index = Some(step_index);
        self
    }

    pub fn with_call_job(mut self, call_job_id: impl Into<String>) -> Self {
        self.call_job_id = Some(call_job_id.into());
        self
    }

    pub fn with_callee(mut self, callee_workflow_path: impl Into<String>) -> Self {
        self.callee_workflow_path = Some(callee_workflow_path.into());
        self
    }
}

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
