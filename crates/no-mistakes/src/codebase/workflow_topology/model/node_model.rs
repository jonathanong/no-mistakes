use super::{JsonScalar, WorkflowCallContract, WorkflowConcurrency};
use crate::codebase::ci_graph::permissions::ResolvedPermissions;
use crate::codebase::workflow_topology::artifact_types::ArtifactDeclaration;
use crate::codebase::workflow_topology::value_primitives::OrderedJson;
use serde::Serialize;
use std::collections::BTreeMap;

/// A parsed workflow file.
///
/// Existing fields retain the standalone TS engine's object-literal order.
/// Enriched schema-v1 fields are appended so the legacy serialized prefix
/// remains stable.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowNode {
    pub id: String,
    pub path: String,
    pub name: String,
    pub callable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_call: Option<WorkflowCallContract>,
    pub triggers: Vec<super::WorkflowTrigger>,
    pub job_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<WorkflowConcurrency>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_references: Option<Vec<String>>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with: Option<BTreeMap<String, JsonScalar>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_references: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum JobKind {
    Job,
    MatrixTemplate,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(untagged)]
pub enum WorkflowRunsOn {
    Label(String),
    Labels(Vec<String>),
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_minutes: Option<serde_json::Number>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runs_on: Option<WorkflowRunsOn>,
    pub permissions: ResolvedPermissions,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_references: Option<Vec<String>>,
}
