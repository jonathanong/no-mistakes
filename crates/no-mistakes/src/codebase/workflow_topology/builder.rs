use super::{
    artifact_resolver, call_contract_diagnostics, graph_diagnostics, model, parse, topology_graph,
    topology_identifiers, workflow_run_graph,
};
use crate::codebase::ci_graph::relative_slash;
use crate::codebase::ci_workflows::ParsedWorkflowSet;
use crate::config::v2::schema::CiConfig;
use std::collections::HashMap;
use std::path::Path;

/// Build a topology graph from workflow documents loaded once by the caller.
///
/// This keeps `ci topology`, legacy CI analysis, and canonical dependency
/// graph producers on a single read/deserialization result per workflow.
pub fn load_workflow_topology_from_parsed(
    root: &Path,
    ci: &CiConfig,
    parsed: &ParsedWorkflowSet,
    workflow_filters: &[String],
) -> model::WorkflowTopology {
    // The same `root.join(dir)` + normalize + relative-slash round trip
    // discovery itself applies to `ci.workflow_dirs` above — so a
    // `--workflow` filter's directory comparisons (`normalize_workflow_filter`)
    // see the exact spelling every discovered workflow's path actually uses
    // (e.g. a configured `./ci-pipelines` or `../external-workflows` matches
    // the discovered `ci-pipelines`/`../external-workflows`, not the raw
    // config string).
    let workflow_dirs: Vec<String> = ci
        .workflow_dirs
        .iter()
        .map(|dir| {
            relative_slash(
                root,
                &crate::codebase::ts_resolver::normalize_path(&root.join(dir)),
            )
        })
        .collect();

    let mut workflows = Vec::new();
    let mut jobs = Vec::new();
    let mut edges = Vec::new();
    let mut diagnostics = Vec::new();
    let mut output_references = Vec::new();

    for document in &parsed.documents {
        let parsed = match &document.value {
            Ok(value) => {
                parse::parse_workflow_value(value, &document.path, document.source.as_deref())
            }
            Err(error) => parse::malformed_workflow(&document.path, &error.message),
        };
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

    let artifact_edges =
        artifact_resolver::resolve_artifact_graph(&workflows, &jobs, &edges, &mut diagnostics);
    edges.extend(
        artifact_edges
            .into_iter()
            .map(model::WorkflowTopologyEdge::Artifact),
    );

    graph_diagnostics::diagnose_job_graph(&jobs, &edges, &mut diagnostics);
    graph_diagnostics::diagnose_duplicate_workflow_names(&workflows, &mut diagnostics);
    topology_graph::diagnose_workflow_filters(
        workflow_filters,
        &workflow_by_path,
        &mut diagnostics,
        &workflow_dirs,
    );

    let selected = topology_graph::select_workflow_paths(
        workflow_filters,
        &workflow_by_path,
        &edges,
        &workflow_dirs,
    );

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
