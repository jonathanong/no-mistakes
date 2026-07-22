//! The four typed edge kinds in the topology graph, split out of
//! [`super`] to stay under the crate's per-file line limit. Re-exported by
//! [`super`] so every existing `model::WorkflowCallEdge`-style path
//! elsewhere in the crate keeps working unchanged.

use super::super::artifact_types::ArtifactEdge;
use serde::Serialize;
use std::collections::BTreeMap;

use super::JsonScalar;

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
