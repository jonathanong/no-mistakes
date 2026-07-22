//! Occurrence/adjacency bookkeeping for [`super::expand_workflow`], split
//! out of the parent file to stay under the crate's per-file line limit.

use super::super::artifact_resolution_types::{ArtifactOccurrence, ArtifactRunContext};
use super::super::model::{WorkflowJobNode, WorkflowTopologyEdge};
use super::super::topology_identifiers::workflow_path_from_id;
use super::{empty_unit, Unit, ARTIFACT_RUN_OCCURRENCE_LIMIT};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// The `needs` edges whose origin belongs to `path`, wiring a same-run
/// precedence arc from every exit of each edge's `from` job to every entry
/// of its `to` job along the way.
pub(super) fn topology_needs_for_workflow(
    path: &str,
    workflow_units: &HashMap<String, Unit>,
    context: &mut ArtifactRunContext,
    topology_edges: &[WorkflowTopologyEdge],
) -> Vec<(String, String)> {
    let edges: Vec<(String, String)> = topology_edges
        .iter()
        .filter_map(|edge| match edge {
            WorkflowTopologyEdge::Needs(needs) if workflow_path_from_id(&needs.from) == path => {
                Some((needs.from.clone(), needs.to.clone()))
            }
            _ => None,
        })
        .collect();
    for (from, to) in &edges {
        let exits: Vec<String> = workflow_units
            .get(from)
            .map(|unit| unit.exits.iter().cloned().collect())
            .unwrap_or_default();
        let entries: Vec<String> = workflow_units
            .get(to)
            .map(|unit| unit.entries.iter().cloned().collect())
            .unwrap_or_default();
        for from_id in &exits {
            for to_id in &entries {
                add_arc(context, from_id, to_id);
            }
        }
    }
    edges
}

pub(super) fn add_occurrence(
    context: &mut ArtifactRunContext,
    invocation: &str,
    job: &WorkflowJobNode,
    inherited_conditional: bool,
    opaque: bool,
    invocation_count: Option<u32>,
) -> Unit {
    if !context.complete {
        return empty_unit();
    }
    if context.occurrences.len() >= ARTIFACT_RUN_OCCURRENCE_LIMIT {
        context.complete = false;
        return empty_unit();
    }
    let id = format!("{invocation}|{}", job.id);
    context.occurrences.push(Rc::new(ArtifactOccurrence {
        id: id.clone(),
        job: job.clone(),
        inherited_conditional,
        opaque,
        invocation_count,
    }));
    context.adjacency.insert(id.clone(), HashSet::new());
    Unit {
        entries: HashSet::from([id.clone()]),
        exits: HashSet::from([id]),
    }
}

pub(super) fn multiply_counts(left: Option<u32>, right: Option<u32>) -> Option<u32> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left * right),
        _ => None,
    }
}

/// No-op if `from` has no adjacency entry yet — mirrors the TS engine's
/// `context.adjacency.get(from)?.add(to)`. In practice every id that
/// reaches here was created by [`add_occurrence`], which always registers
/// one; the only way it could be missing is `context.complete` having
/// flipped to `false` mid-expansion, in which case the whole context (and
/// its adjacency) is discarded by the caller regardless.
fn add_arc(context: &mut ArtifactRunContext, from: &str, to: &str) {
    if let Some(set) = context.adjacency.get_mut(from) {
        set.insert(to.to_string());
    }
}
