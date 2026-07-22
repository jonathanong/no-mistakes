//! Reusable-workflow call diagnostics, `--workflow` filter selection, and
//! the final global sort, ported from `topology-graph.mts`.
//!
//! `selectWorkflowPaths` in the original engine calls into the frozen
//! `WorkflowTopologyIndex`'s `transitiveCalleeWorkflowPaths`, which this
//! Rust port does not build (the query index stays JS-only — see the
//! module docs on [`super`]). [`select_workflow_paths`] below inlines the
//! one traversal it actually needs: a transitive closure over local-call
//! edges, built directly from the edge list already at hand.

use super::model;
use super::posix_path;
use super::topology_identifiers;
use std::collections::{HashMap, HashSet};

pub const WORKFLOWS_DIRECTORY: &str = ".github/workflows";

pub fn diagnose_calls(
    workflow_by_path: &HashMap<String, model::WorkflowNode>,
    edges: &[model::WorkflowTopologyEdge],
    diagnostics: &mut Vec<model::WorkflowTopologyDiagnostic>,
) {
    for edge in edges {
        let model::WorkflowTopologyEdge::Calls(call) = edge else {
            continue;
        };
        let (true, Some(to)) = (call.local, &call.to) else {
            continue;
        };
        let caller_path = topology_identifiers::workflow_path_from_id(&call.from);
        match workflow_by_path.get(to) {
            None => diagnostics.push(
                model::WorkflowTopologyDiagnostic::new(
                    model::DiagnosticCode::MissingLocalWorkflow,
                    format!("{} calls missing workflow {to}", call.from),
                    caller_path,
                )
                .with_job(&call.from),
            ),
            Some(callee) if !callee.callable => diagnostics.push(
                model::WorkflowTopologyDiagnostic::new(
                    model::DiagnosticCode::NonCallableWorkflow,
                    format!(
                        "{} calls {to}, which does not declare workflow_call",
                        call.from
                    ),
                    caller_path,
                )
                .with_job(&call.from),
            ),
            _ => {}
        }
    }
}

pub fn diagnose_call_cycles(
    workflows: &[model::WorkflowNode],
    edges: &[model::WorkflowTopologyEdge],
    diagnostics: &mut Vec<model::WorkflowTopologyDiagnostic>,
) {
    let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
    for edge in edges {
        if let model::WorkflowTopologyEdge::Calls(call) = edge {
            if call.local {
                if let Some(to) = &call.to {
                    let from = topology_identifiers::workflow_path_from_id(&call.from).to_string();
                    adjacency.entry(from).or_default().push(to.clone());
                }
            }
        }
    }
    for targets in adjacency.values_mut() {
        targets.sort();
    }

    let mut active: HashSet<String> = HashSet::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut reported: HashSet<String> = HashSet::new();
    for workflow in workflows {
        visit_call_cycle(
            &workflow.path,
            &[],
            &adjacency,
            &mut active,
            &mut visited,
            &mut reported,
            diagnostics,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn visit_call_cycle(
    path: &str,
    stack: &[String],
    adjacency: &HashMap<String, Vec<String>>,
    active: &mut HashSet<String>,
    visited: &mut HashSet<String>,
    reported: &mut HashSet<String>,
    diagnostics: &mut Vec<model::WorkflowTopologyDiagnostic>,
) {
    if active.contains(path) {
        let start = stack.iter().position(|node| node == path).unwrap_or(0);
        let mut cycle: Vec<String> = stack[start..].to_vec();
        cycle.push(path.to_string());
        let mut unique: Vec<String> = cycle
            .iter()
            .cloned()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        unique.sort();
        if reported.insert(unique.join("|")) {
            diagnostics.push(model::WorkflowTopologyDiagnostic::new(
                model::DiagnosticCode::WorkflowCallCycle,
                format!("reusable workflow call cycle: {}", cycle.join(" -> ")),
                path,
            ));
        }
        return;
    }
    if visited.contains(path) {
        return;
    }
    active.insert(path.to_string());
    if let Some(targets) = adjacency.get(path) {
        for target in targets {
            let mut next_stack = stack.to_vec();
            next_stack.push(path.to_string());
            visit_call_cycle(
                target,
                &next_stack,
                adjacency,
                active,
                visited,
                reported,
                diagnostics,
            );
        }
    }
    active.remove(path);
    visited.insert(path.to_string());
}

/// Every workflow when `requested` is empty; otherwise the normalized
/// requested set plus, for each requested workflow that exists, its
/// transitive local reusable-workflow callees.
pub fn select_workflow_paths(
    requested: &[String],
    workflows: &HashMap<String, model::WorkflowNode>,
    edges: &[model::WorkflowTopologyEdge],
) -> HashSet<String> {
    if requested.is_empty() {
        return workflows.keys().cloned().collect();
    }
    let mut selected: HashSet<String> = requested
        .iter()
        .filter_map(|path| normalize_workflow_filter(path))
        .collect();
    let initially_selected: Vec<String> = selected.iter().cloned().collect();

    let mut call_adjacency: HashMap<String, Vec<String>> = HashMap::new();
    for edge in edges {
        if let model::WorkflowTopologyEdge::Calls(call) = edge {
            if call.local {
                if let Some(to) = &call.to {
                    let from = topology_identifiers::workflow_path_from_id(&call.from).to_string();
                    call_adjacency.entry(from).or_default().push(to.clone());
                }
            }
        }
    }

    for path in initially_selected {
        if !workflows.contains_key(&path) {
            continue;
        }
        selected.extend(transitive_local_callees(&path, &call_adjacency));
    }
    selected
}

/// Transitive local-call closure starting from (but excluding) `start`,
/// including when a call cycle leads back to `start` itself.
fn transitive_local_callees(
    start: &str,
    adjacency: &HashMap<String, Vec<String>>,
) -> HashSet<String> {
    let mut result = HashSet::new();
    let mut seen: HashSet<String> = HashSet::from([start.to_string()]);
    let mut stack = vec![start.to_string()];
    while let Some(current) = stack.pop() {
        let Some(targets) = adjacency.get(&current) else {
            continue;
        };
        for target in targets {
            if seen.insert(target.clone()) {
                result.insert(target.clone());
                stack.push(target.clone());
            }
        }
    }
    result
}

pub fn diagnose_workflow_filters(
    requested: &[String],
    workflows: &HashMap<String, model::WorkflowNode>,
    diagnostics: &mut Vec<model::WorkflowTopologyDiagnostic>,
) {
    for requested_path in requested {
        let Some(path) = normalize_workflow_filter(requested_path) else {
            diagnostics.push(model::WorkflowTopologyDiagnostic::new(
                model::DiagnosticCode::InvalidWorkflowFilter,
                format!(
                    "workflow filter must be a basename or a path inside {WORKFLOWS_DIRECTORY}: {requested_path}"
                ),
                requested_path.clone(),
            ));
            continue;
        };
        if workflows.contains_key(&path) {
            continue;
        }
        diagnostics.push(model::WorkflowTopologyDiagnostic::new(
            model::DiagnosticCode::UnknownWorkflowFilter,
            format!("workflow filter does not match a repository workflow: {requested_path}"),
            path,
        ));
    }
}

/// Normalizes a `--workflow` filter to a `.github/workflows/<file>` path,
/// or `None` when it's absolute, escapes the workflows directory via `..`,
/// or (once it contains a `/`) doesn't resolve to a direct child of
/// `.github/workflows`. A bare basename is joined onto the workflows
/// directory.
fn normalize_workflow_filter(path: &str) -> Option<String> {
    let slashed = path.replace('\\', "/");
    let candidate = strip_leading_dot_slashes(&slashed);
    if candidate.is_empty()
        || candidate.starts_with('/')
        || candidate.split('/').any(|segment| segment == "..")
    {
        return None;
    }
    if !candidate.contains('/') {
        return if candidate == "." || candidate == ".." {
            None
        } else {
            Some(format!("{WORKFLOWS_DIRECTORY}/{candidate}"))
        };
    }
    let normalized = posix_path::normalize(candidate);
    (posix_path::dirname(&normalized) == WORKFLOWS_DIRECTORY).then_some(normalized)
}

fn strip_leading_dot_slashes(path: &str) -> &str {
    let mut rest = path;
    while let Some(stripped) = rest.strip_prefix("./") {
        rest = stripped;
    }
    rest
}

/// `calls` edges belong to the selection whenever their origin does,
/// regardless of whether their (possibly absent, for remote calls)
/// destination is selected — the destination is only meaningfully checked
/// for the other three edge kinds.
pub fn edge_belongs_to_selection(
    edge: &model::WorkflowTopologyEdge,
    selected: &HashSet<String>,
) -> bool {
    if !selected.contains(topology_identifiers::workflow_path_from_id(edge.from())) {
        return false;
    }
    matches!(edge, model::WorkflowTopologyEdge::Calls(_))
        || selected.contains(topology_identifiers::workflow_path_from_id(
            edge.to().unwrap_or(""),
        ))
}

/// Global deterministic sort: workflows and jobs by id, edges by a
/// per-kind composite key, diagnostics by
/// `workflowPath|code|jobId|message`. This ordering — not push order — is
/// what makes the serialized JSON stable.
pub fn sort_topology(mut topology: model::WorkflowTopology) -> model::WorkflowTopology {
    topology
        .workflows
        .sort_by(|left, right| left.id.cmp(&right.id));
    topology.jobs.sort_by(|left, right| left.id.cmp(&right.id));
    topology.edges.sort_by_key(edge_key);
    topology.diagnostics.sort_by_key(diagnostic_key);
    topology
}

fn edge_key(edge: &model::WorkflowTopologyEdge) -> String {
    match edge {
        model::WorkflowTopologyEdge::Needs(edge) => format!("needs|{}|{}", edge.from, edge.to),
        model::WorkflowTopologyEdge::WorkflowRun(edge) => {
            format!("workflow-run|{}|{}", edge.from, edge.to)
        }
        model::WorkflowTopologyEdge::Artifact(edge) => format!(
            "artifact|{}|{}|{}|{}|{}|{}",
            edge.from,
            edge.producer_step,
            edge.to,
            edge.consumer_step,
            edge.name,
            edge.match_kind.as_str()
        ),
        model::WorkflowTopologyEdge::Calls(edge) => format!("calls|{}|{}", edge.from, edge.target),
    }
}

fn diagnostic_key(diagnostic: &model::WorkflowTopologyDiagnostic) -> String {
    format!(
        "{}|{}|{}|{}",
        diagnostic.workflow_path,
        diagnostic.code.as_str(),
        diagnostic.job_id.as_deref().unwrap_or(""),
        diagnostic.message,
    )
}
