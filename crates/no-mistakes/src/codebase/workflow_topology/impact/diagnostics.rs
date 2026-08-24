use super::super::model::{DiagnosticCode, WorkflowTopology, WorkflowTopologyDiagnostic};
use super::workflow_graph::root_callers;
use super::{CiTopologyImpactDiagnostic, CiTopologyImpactDiagnosticScope};
use std::collections::BTreeSet;

pub(super) fn topology_diagnostic(
    diagnostic: &WorkflowTopologyDiagnostic,
    entry: &str,
    base: &WorkflowTopology,
    head: &WorkflowTopology,
) -> CiTopologyImpactDiagnostic {
    let mut endpoints = BTreeSet::from([diagnostic.workflow_path.clone()]);
    if let Some(callee) = &diagnostic.callee_workflow_path {
        endpoints.insert(callee.clone());
    }
    // Every implicated endpoint needs its own entry-root proof. A union query
    // can mask one unbound endpoint behind another bound endpoint.
    let endpoint_roots = endpoints
        .iter()
        .map(|endpoint| root_callers(entry, &BTreeSet::from([endpoint.clone()]), base, head))
        .collect::<Vec<_>>();
    let mut roots = endpoint_roots
        .iter()
        .flatten()
        .cloned()
        .collect::<BTreeSet<_>>();
    let entry_job_is_bound = diagnostic.workflow_path == entry
        && diagnostic.job_id.as_ref().is_some_and(|job_id| {
            base.jobs
                .iter()
                .chain(&head.jobs)
                .any(|job| job.id == *job_id && job.workflow_id == entry)
        });
    if let Some(job_id) = diagnostic.job_id.as_ref().filter(|_| entry_job_is_bound) {
        roots.insert(job_id.clone());
    }
    let localized = !roots.is_empty()
        && !requires_unbound_global(diagnostic.code)
        && endpoints
            .iter()
            .zip(&endpoint_roots)
            .all(|(endpoint, roots)| {
                (endpoint == entry && entry_job_is_bound) || !roots.is_empty()
            })
        && endpoints.iter().all(|endpoint| {
            endpoint == entry
                || base
                    .workflows
                    .iter()
                    .chain(&head.workflows)
                    .any(|workflow| workflow.path == *endpoint)
        });
    CiTopologyImpactDiagnostic {
        code: diagnostic.code.as_str().to_string(),
        message: diagnostic.message.clone(),
        workflow_path: Some(diagnostic.workflow_path.clone()),
        scope: if localized {
            CiTopologyImpactDiagnosticScope::Localized
        } else {
            CiTopologyImpactDiagnosticScope::Global
        },
        root_job_ids: localized.then(|| roots.into_iter().collect()),
    }
}

pub(super) fn requires_unbound_global(code: DiagnosticCode) -> bool {
    matches!(
        code,
        DiagnosticCode::MalformedWorkflow
            | DiagnosticCode::DuplicateWorkflowName
            | DiagnosticCode::MissingNeedsDependency
            | DiagnosticCode::MissingLocalWorkflow
            | DiagnosticCode::NonCallableWorkflow
            | DiagnosticCode::MissingWorkflowRunSource
            | DiagnosticCode::AmbiguousWorkflowRunSource
            | DiagnosticCode::WorkflowRunCycle
            | DiagnosticCode::WorkflowRunChainLimit
            | DiagnosticCode::UnknownWorkflowFilter
            | DiagnosticCode::InvalidWorkflowFilter
            | DiagnosticCode::MissingArtifactProducer
            | DiagnosticCode::AmbiguousArtifactProducer
            | DiagnosticCode::ArtifactResolutionLimit
    )
}
