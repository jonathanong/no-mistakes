//! Runtime-only types for same-run artifact producer/consumer resolution,
//! ported from `artifact-resolution-types.mts`. None of these serialize —
//! they exist only while [`super::artifact_resolver::resolve_artifact_graph`]
//! walks one root workflow's expanded job graph.

use super::artifact_types::{ArtifactEdge, ArtifactUploadDeclaration};
use super::model::{WorkflowJobNode, WorkflowTopologyDiagnostic};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// One occurrence of a job within a single root workflow's expanded run: a
/// job reached through a local reusable-workflow call is expanded into the
/// caller's run rather than treated as a separate root, so the same
/// workflow file can contribute more than one occurrence (or none, if it's
/// never reached from any root). `job` is cloned once at occurrence
/// creation and shared via [`Rc`] from then on — candidates and reachability
/// lookups clone the handle, never the underlying job.
#[derive(Debug)]
pub struct ArtifactOccurrence {
    pub id: String,
    pub job: WorkflowJobNode,
    pub inherited_conditional: bool,
    pub opaque: bool,
    pub invocation_count: Option<u32>,
}

/// One root workflow's expanded same-run job graph: every occurrence
/// reachable from that root, a `needs`-derived adjacency for "can X precede
/// Y" reachability queries, and a memoization cache for those queries.
/// `complete` flips to `false` (and further expansion stops) once
/// [`super::artifact_run_context::ARTIFACT_RUN_OCCURRENCE_LIMIT`] is hit.
#[derive(Debug, Default)]
pub struct ArtifactRunContext {
    pub occurrences: Vec<Rc<ArtifactOccurrence>>,
    pub adjacency: HashMap<String, HashSet<String>>,
    pub reachability_cache: RefCell<HashMap<String, HashSet<String>>>,
    pub complete: bool,
}

/// An upload step eligible to produce an artifact for a given download
/// step: the occurrence and step index it runs at, plus its parsed upload
/// declaration.
#[derive(Debug)]
pub struct ArtifactCandidate {
    pub occurrence: Rc<ArtifactOccurrence>,
    pub step_index: u32,
    pub upload: ArtifactUploadDeclaration,
}

/// The result of resolving one download step: zero or more producer edges,
/// or (mutually exclusive with edges, in practice) a single diagnostic.
#[derive(Debug, Default)]
pub struct ArtifactResolution {
    pub edges: Vec<ArtifactEdge>,
    pub diagnostic: Option<WorkflowTopologyDiagnostic>,
}
