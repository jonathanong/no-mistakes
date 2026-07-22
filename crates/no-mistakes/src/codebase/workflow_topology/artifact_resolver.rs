//! Top-level same-run artifact dataflow resolution, ported from
//! `artifact-resolver.mts`. Expands every "root" workflow (one not solely
//! reachable via `workflow_call`) into its own run context, resolves every
//! download step within it, then keeps only the edges/diagnostics that
//! agree across every root that reaches a given job — a job reachable from
//! more than one root can disagree on what it downloads (e.g. one caller
//! supplies a producer, another doesn't), and an edge or diagnostic that
//! isn't true for every invocation isn't true at all.

use super::artifact_download_resolver::resolve_artifact_download;
use super::artifact_resolution_helpers::{diagnostic_key, edge_key, edge_set_key, unique_edges};
use super::artifact_resolution_types::ArtifactResolution;
use super::artifact_run_context::{build_artifact_run_context, ARTIFACT_RUN_OCCURRENCE_LIMIT};
use super::artifact_types::{ArtifactDeclaration, ArtifactEdge};
use super::model::{
    DiagnosticCode, WorkflowCallEdge, WorkflowJobNode, WorkflowNode, WorkflowTopologyDiagnostic,
    WorkflowTopologyEdge,
};
use std::collections::HashMap;

pub fn resolve_artifact_graph(
    workflows: &[WorkflowNode],
    jobs: &[WorkflowJobNode],
    topology_edges: &[WorkflowTopologyEdge],
    diagnostics: &mut Vec<WorkflowTopologyDiagnostic>,
) -> Vec<ArtifactEdge> {
    let workflow_by_path: HashMap<String, WorkflowNode> = workflows
        .iter()
        .map(|workflow| (workflow.path.clone(), workflow.clone()))
        .collect();
    let mut jobs_by_workflow: HashMap<String, Vec<WorkflowJobNode>> = HashMap::new();
    for job in jobs {
        jobs_by_workflow
            .entry(job.workflow_id.clone())
            .or_default()
            .push(job.clone());
    }
    let call_by_job: HashMap<String, WorkflowCallEdge> = topology_edges
        .iter()
        .filter_map(|edge| match edge {
            WorkflowTopologyEdge::Calls(call) => Some((call.from.clone(), call.clone())),
            _ => None,
        })
        .collect();
    // A workflow that is *only* triggered by `workflow_call` never runs on
    // its own — it always gets expanded from whichever workflow calls it.
    let roots: Vec<&WorkflowNode> = workflows
        .iter()
        .filter(|workflow| {
            !workflow.callable
                || workflow
                    .triggers
                    .iter()
                    .any(|trigger| trigger.event != "workflow_call")
        })
        .collect();

    let mut resolutions: HashMap<String, Vec<ArtifactResolution>> = HashMap::new();
    let mut order: Vec<String> = Vec::new();
    let mut truncated_roots: Vec<String> = Vec::new();

    for root in &roots {
        let context = build_artifact_run_context(
            &root.path,
            &workflow_by_path,
            &jobs_by_workflow,
            &call_by_job,
            topology_edges,
        );
        if !context.complete {
            truncated_roots.push(root.path.clone());
            continue;
        }
        for occurrence in &context.occurrences {
            for step in &occurrence.job.steps {
                let Some(ArtifactDeclaration::Download(download)) = &step.artifact else {
                    continue;
                };
                let key = format!("{}:{}", occurrence.job.id, step.index);
                let result = resolve_artifact_download(&context, occurrence, step.index, download);
                if !resolutions.contains_key(&key) {
                    order.push(key.clone());
                }
                resolutions.entry(key).or_default().push(result);
            }
        }
    }

    // Any truncated root invalidates the whole graph: we can't tell which
    // of its jobs would have contributed edges, so a partial answer would
    // be misleading. The diagnostic alone still tells the user why.
    if !truncated_roots.is_empty() {
        diagnostics.extend(truncated_roots.iter().map(|workflow_path| {
            WorkflowTopologyDiagnostic::new(
                DiagnosticCode::ArtifactResolutionLimit,
                format!(
                    "{workflow_path} exceeds the {ARTIFACT_RUN_OCCURRENCE_LIMIT}-occurrence local reusable-workflow artifact resolution limit"
                ),
                workflow_path.clone(),
            )
        }));
        return Vec::new();
    }

    let mut artifact_edges: Vec<ArtifactEdge> = Vec::new();
    for key in &order {
        let results = &resolutions[key];
        artifact_edges.extend(edges_shared_across_every_invocation(results));
        if let Some(diagnostic) = diagnostic_shared_across_every_invocation(results) {
            diagnostics.push(diagnostic);
        }
    }
    let mut edges = unique_edges(&artifact_edges);
    edges.sort_by_key(edge_key);
    edges
}

fn edges_shared_across_every_invocation(results: &[ArtifactResolution]) -> Vec<ArtifactEdge> {
    let first = results
        .first()
        .expect("resolutions entries are always pushed to before being read");
    let first_edges = unique_edges(&first.edges);
    let first_key = edge_set_key(&first.edges);
    let all_match = results
        .iter()
        .all(|result| edge_set_key(&result.edges) == first_key);
    if all_match {
        first_edges
    } else {
        Vec::new()
    }
}

fn diagnostic_shared_across_every_invocation(
    results: &[ArtifactResolution],
) -> Option<WorkflowTopologyDiagnostic> {
    let first = results
        .first()
        .expect("resolutions entries are always pushed to before being read");
    let first_diagnostic = first.diagnostic.as_ref()?;
    let first_key = diagnostic_key(Some(first_diagnostic));
    let all_match = results
        .iter()
        .all(|result| diagnostic_key(result.diagnostic.as_ref()) == first_key);
    all_match.then(|| first_diagnostic.clone())
}
