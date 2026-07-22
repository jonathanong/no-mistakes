//! GitHub Actions workflow topology graph (`no-mistakes ci topology`).
//!
//! Parses `.github/workflows/*.{yml,yaml}` into a typed graph — workflows,
//! jobs, and edges for `needs` control flow, reusable-workflow calls,
//! `workflow_run` subscriptions, and (once a later port wave lands) same-run
//! artifact dataflow — plus structured diagnostics for malformed, dangling,
//! cyclic, or contract-violating workflow definitions.
//!
//! This is a faithful Rust port of a standalone TypeScript engine's
//! schema-v1 model. The serialized [`model::WorkflowTopology`] JSON shape
//! (field names, field ORDER, and array/diagnostic sort order) is a
//! stability contract downstream consumers snapshot-diff against — see the
//! field-order notes in `model.rs` before changing anything there.
//!
//! Workflow directories come from [`CiConfig`] (shared with
//! [`super::ci_graph`]), defaulting to `.github/workflows`; discovery reuses
//! [`super::ci_graph::discover_workflow_files_from_snapshot`] rather than
//! walking the filesystem again.
//!
//! The original TS engine also ships a `WorkflowTopologyIndex` — a frozen
//! query object with ~20 traversal methods (transitive callers/callees,
//! `workflow_run` closures, artifact producer/consumer lookups). That index
//! is intentionally **not** ported here: it doesn't cross an N-API boundary
//! cleanly (it returns closures over frozen `Map`s), and its only consumer
//! today rebuilds a thin JS-side equivalent from this module's serialized
//! JSON. [`topology_graph::select_workflow_paths`] inlines the one
//! traversal (`--workflow` filter transitive-callee closure) this module
//! itself needs from that index.

pub mod artifact_types;
pub mod call_contract_diagnostics;
pub mod case_insensitive_lookup;
pub mod expression_references;
pub mod graph_algorithms;
pub mod graph_diagnostics;
pub mod model;
pub mod parse;
pub mod posix_path;
pub mod render_json;
pub mod render_mermaid;
pub mod topology_graph;
pub mod topology_identifiers;
pub mod value_primitives;
pub mod workflow_run_graph;
pub mod workflow_values;

#[cfg(test)]
mod tests;

use crate::codebase::ci_graph::{discover_workflow_files_from_snapshot, relative_slash};
use crate::codebase::ts_source::VisiblePathSnapshot;
use crate::config::v2::schema::CiConfig;
use std::collections::HashMap;
use std::path::Path;

/// Loads the full workflow topology graph for a repository, building its
/// own visibility snapshot. Mirrors [`super::ci_graph::WorkflowSet::load`]'s
/// ergonomics; a caller that already has a snapshot (e.g. because it also
/// needs one to load config) should use
/// [`load_workflow_topology_from_snapshot`] instead to avoid discovering
/// the file universe twice in one invocation.
pub fn load_workflow_topology(
    root: &Path,
    ci: &CiConfig,
    workflow_filters: &[String],
) -> model::WorkflowTopology {
    let snapshot = VisiblePathSnapshot::new(root);
    load_workflow_topology_from_snapshot(root, ci, &snapshot, workflow_filters)
}

/// Parse-once-per-invocation variant of [`load_workflow_topology`] that
/// reuses a request-scoped [`VisiblePathSnapshot`] instead of building its
/// own. Discovers every workflow file under `ci.workflow_dirs`, parses each
/// one, then runs the full cross-file diagnostic and edge-resolution
/// pipeline in the same order as the TS engine's `loadWorkflowTopology`
/// (parse all → the `workflow_run` graph → local-call diagnostics →
/// call-contract diagnostics → call cycles → artifact graph → job graph →
/// duplicate names → `--workflow` filter validation → select + filter +
/// sort).
#[doc(hidden)]
pub fn load_workflow_topology_from_snapshot(
    root: &Path,
    ci: &CiConfig,
    snapshot: &VisiblePathSnapshot,
    workflow_filters: &[String],
) -> model::WorkflowTopology {
    let mut paths: Vec<String> = discover_workflow_files_from_snapshot(root, ci, snapshot)
        .into_iter()
        .map(|absolute| relative_slash(root, &absolute))
        .collect();
    paths.sort();

    let mut workflows = Vec::new();
    let mut jobs = Vec::new();
    let mut edges = Vec::new();
    let mut diagnostics = Vec::new();
    let mut output_references = Vec::new();

    for path in &paths {
        let parsed = parse::parse_workflow_file(root, path);
        workflows.push(parsed.node);
        jobs.extend(parsed.jobs);
        edges.extend(parsed.edges);
        diagnostics.extend(parsed.diagnostics);
        output_references.extend(parsed.output_references);
    }

    let workflow_by_path: HashMap<String, model::WorkflowNode> = workflows
        .iter()
        .map(|workflow| (workflow.path.clone(), workflow.clone()))
        .collect();

    edges.extend(workflow_run_graph::resolve_workflow_run_graph(
        &workflows,
        &mut diagnostics,
    ));
    topology_graph::diagnose_calls(&workflow_by_path, &edges, &mut diagnostics);
    call_contract_diagnostics::diagnose_workflow_call_contracts(
        &workflows,
        &jobs,
        &edges,
        &output_references,
        &mut diagnostics,
    );
    topology_graph::diagnose_call_cycles(&workflows, &edges, &mut diagnostics);

    // TODO(workflow-topology-artifacts): replace with the real
    // artifact-dataflow resolver (upload/download-artifact edges plus
    // missing/ambiguous/resolution-limit diagnostics). Until then this
    // contributes nothing, and every `WorkflowStep::artifact` is `None`.
    let (artifact_edges, artifact_diagnostics) =
        resolve_artifact_graph_stub(&workflows, &jobs, &edges, &diagnostics);
    edges.extend(artifact_edges);
    diagnostics.extend(artifact_diagnostics);

    graph_diagnostics::diagnose_job_graph(&jobs, &edges, &mut diagnostics);
    graph_diagnostics::diagnose_duplicate_workflow_names(&workflows, &mut diagnostics);
    topology_graph::diagnose_workflow_filters(
        workflow_filters,
        &workflow_by_path,
        &mut diagnostics,
    );

    let selected =
        topology_graph::select_workflow_paths(workflow_filters, &workflow_by_path, &edges);

    let topology = model::WorkflowTopology {
        schema_version: model::WORKFLOW_TOPOLOGY_SCHEMA_VERSION,
        workflows: workflows
            .into_iter()
            .filter(|workflow| selected.contains(&workflow.path))
            .collect(),
        jobs: jobs
            .into_iter()
            .filter(|job| {
                selected.contains(topology_identifiers::workflow_path_from_id(
                    &job.workflow_id,
                ))
            })
            .collect(),
        edges: edges
            .into_iter()
            .filter(|edge| topology_graph::edge_belongs_to_selection(edge, &selected))
            .collect(),
        diagnostics: diagnostics
            .into_iter()
            .filter(|diagnostic| {
                diagnostic.code == model::DiagnosticCode::InvalidWorkflowFilter
                    || selected.contains(&diagnostic.workflow_path)
            })
            .collect(),
    };

    topology_graph::sort_topology(topology)
}

/// Seam for the artifact-dataflow resolver (a later port wave), mirroring
/// the TS engine's `resolveArtifactGraph(workflows, jobs, edges,
/// diagnostics)` call site so wiring in the real resolver only touches
/// this one function.
fn resolve_artifact_graph_stub(
    _workflows: &[model::WorkflowNode],
    _jobs: &[model::WorkflowJobNode],
    _edges: &[model::WorkflowTopologyEdge],
    _diagnostics: &[model::WorkflowTopologyDiagnostic],
) -> (
    Vec<model::WorkflowTopologyEdge>,
    Vec<model::WorkflowTopologyDiagnostic>,
) {
    (Vec::new(), Vec::new())
}
