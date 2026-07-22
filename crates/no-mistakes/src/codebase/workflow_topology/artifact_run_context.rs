//! Expands one root workflow into its full same-run job graph — following
//! local reusable-workflow calls into the caller's run instead of treating
//! them as separate roots — ported from `artifact-run-context.mts`.

use super::artifact_resolution_types::ArtifactRunContext;
use super::artifact_values::static_matrix_instance_count;
use super::model::{WorkflowCallEdge, WorkflowJobNode, WorkflowNode, WorkflowTopologyEdge};
use occurrence::{add_occurrence, multiply_counts, topology_needs_for_workflow};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

mod occurrence;

/// Hard cap on same-run occurrences a single root may expand to. Exceeding
/// it (e.g. an exponentially branching local-call DAG) marks the context
/// incomplete; the caller discards it and emits a single
/// `artifact-resolution-limit` diagnostic instead of a partial graph.
pub const ARTIFACT_RUN_OCCURRENCE_LIMIT: usize = 4096;

/// A workflow's (or a job's) entry/exit occurrence-id frontier, used to
/// wire cross-workflow (`needs`, or caller/callee boundary) adjacency arcs
/// without re-deriving the whole job DAG at every boundary.
struct Unit {
    entries: HashSet<String>,
    exits: HashSet<String>,
}

fn empty_unit() -> Unit {
    Unit {
        entries: HashSet::new(),
        exits: HashSet::new(),
    }
}

/// The parsed-graph lookup tables `expand_workflow` recurses over. Bundled
/// into one struct (rather than four separate parameters) purely to stay
/// under the crate's argument-count limit — every field is a read-only
/// borrow, unchanged for the lifetime of one [`build_artifact_run_context`]
/// call.
struct WorkflowGraphLookups<'a> {
    workflow_by_path: &'a HashMap<String, WorkflowNode>,
    jobs_by_workflow: &'a HashMap<String, Vec<WorkflowJobNode>>,
    call_by_job: &'a HashMap<String, WorkflowCallEdge>,
    topology_edges: &'a [WorkflowTopologyEdge],
}

pub fn build_artifact_run_context(
    root_path: &str,
    workflow_by_path: &HashMap<String, WorkflowNode>,
    jobs_by_workflow: &HashMap<String, Vec<WorkflowJobNode>>,
    call_by_job: &HashMap<String, WorkflowCallEdge>,
    topology_edges: &[WorkflowTopologyEdge],
) -> ArtifactRunContext {
    let mut context = ArtifactRunContext {
        occurrences: Vec::new(),
        adjacency: HashMap::new(),
        reachability_cache: RefCell::new(HashMap::new()),
        complete: true,
    };
    let graph = WorkflowGraphLookups {
        workflow_by_path,
        jobs_by_workflow,
        call_by_job,
        topology_edges,
    };
    expand_workflow(
        root_path,
        root_path,
        false,
        Some(1),
        &mut context,
        &[],
        &graph,
    );
    context
}

fn expand_workflow(
    workflow_path: &str,
    invocation: &str,
    inherited_conditional: bool,
    inherited_invocation_count: Option<u32>,
    context: &mut ArtifactRunContext,
    active_paths: &[String],
    graph: &WorkflowGraphLookups,
) -> Unit {
    if !context.complete || active_paths.iter().any(|path| path == workflow_path) {
        return empty_unit();
    }
    let empty_jobs = Vec::new();
    let workflow_jobs = graph
        .jobs_by_workflow
        .get(workflow_path)
        .unwrap_or(&empty_jobs);
    let mut units: HashMap<String, Unit> = HashMap::new();
    for job in workflow_jobs {
        if !context.complete {
            break;
        }
        let conditional = inherited_conditional || job.condition.is_some();
        let call = graph.call_by_job.get(&job.id);
        let callee = call
            .filter(|call| call.local)
            .and_then(|call| call.to.as_deref())
            .and_then(|to| graph.workflow_by_path.get(to));
        let unit = match callee.filter(|callee| callee.callable) {
            Some(callee) => {
                let invocation_count = multiply_counts(
                    inherited_invocation_count,
                    static_matrix_instance_count(job.matrix.as_ref()),
                );
                let mut child_active_paths: Vec<String> = active_paths.to_vec();
                child_active_paths.push(workflow_path.to_string());
                let child = expand_workflow(
                    &callee.path,
                    &format!("{invocation}>{}", job.key),
                    conditional,
                    invocation_count,
                    context,
                    &child_active_paths,
                    graph,
                );
                if !child.entries.is_empty() {
                    child
                } else {
                    add_occurrence(
                        context,
                        invocation,
                        job,
                        conditional,
                        false,
                        inherited_invocation_count,
                    )
                }
            }
            None => add_occurrence(
                context,
                invocation,
                job,
                conditional,
                call.is_some(),
                inherited_invocation_count,
            ),
        };
        units.insert(job.id.clone(), unit);
    }

    let needs_edges =
        topology_needs_for_workflow(workflow_path, &units, context, graph.topology_edges);
    let incoming: HashSet<&str> = needs_edges.iter().map(|(_, to)| to.as_str()).collect();
    let outgoing: HashSet<&str> = needs_edges.iter().map(|(from, _)| from.as_str()).collect();

    Unit {
        entries: workflow_jobs
            .iter()
            .filter(|job| !incoming.contains(job.id.as_str()))
            .flat_map(|job| {
                units
                    .get(&job.id)
                    .map(|unit| unit.entries.iter().cloned())
                    .into_iter()
                    .flatten()
            })
            .collect(),
        exits: workflow_jobs
            .iter()
            .filter(|job| !outgoing.contains(job.id.as_str()))
            .flat_map(|job| {
                units
                    .get(&job.id)
                    .map(|unit| unit.exits.iter().cloned())
                    .into_iter()
                    .flatten()
            })
            .collect(),
    }
}
