//! `secrets:` and output-reference contract checks, split out of [`super`]
//! to stay under the crate's per-file line limit.

use super::super::case_insensitive_lookup::{CaseInsensitiveLookup, Resolution};
use super::super::model;
use super::super::parse::ParsedWorkflowOutputReference;
use super::super::topology_identifiers;
use super::{call_diagnostic, resolved_callable_callee};
use std::collections::{HashMap, HashSet};

pub(super) fn diagnose_secrets(
    edge: &model::WorkflowCallEdge,
    callee: &model::WorkflowNode,
    diagnostics: &mut Vec<model::WorkflowTopologyDiagnostic>,
) {
    let model::WorkflowCallSecretsBinding::Explicit { values } = &edge.bindings.secrets else {
        return; // "inherit" mode skips every secret check
    };
    let Some(contract) = &callee.workflow_call else {
        return;
    };
    let declarations = CaseInsensitiveLookup::new(
        contract
            .secrets
            .iter()
            .map(|(key, value)| (key.clone(), value)),
    );
    let bindings =
        CaseInsensitiveLookup::new(values.iter().map(|(key, value)| (key.clone(), value)));

    for (name, declaration) in declarations.unique_entries() {
        match bindings.resolve(name) {
            Resolution::Missing if declaration.required => {
                diagnostics.push(call_diagnostic(
                    edge,
                    &callee.path,
                    model::DiagnosticCode::MissingWorkflowCallSecret,
                    format!(
                        "{} does not provide required secret {name} to {}",
                        edge.from, callee.path
                    ),
                ));
            }
            _ => {}
        }
    }
    for (name, _declaration) in bindings.unique_entries() {
        if !matches!(declarations.resolve(name), Resolution::Missing) {
            continue;
        }
        diagnostics.push(call_diagnostic(
            edge,
            &callee.path,
            model::DiagnosticCode::UnknownWorkflowCallSecret,
            format!(
                "{} provides unknown secret {name} to {}",
                edge.from, callee.path
            ),
        ));
    }
}

pub(super) fn diagnose_outputs(
    jobs: &[model::WorkflowJobNode],
    call_edges: &[&model::WorkflowCallEdge],
    references: &[ParsedWorkflowOutputReference],
    workflows_by_path: &HashMap<String, &model::WorkflowNode>,
    diagnostics: &mut Vec<model::WorkflowTopologyDiagnostic>,
) {
    let calls_by_job_id: HashMap<&str, &model::WorkflowCallEdge> = call_edges
        .iter()
        .map(|edge| (edge.from.as_str(), *edge))
        .collect();
    let mut jobs_by_workflow: HashMap<&str, Vec<&model::WorkflowJobNode>> = HashMap::new();
    for job in jobs {
        jobs_by_workflow
            .entry(job.workflow_id.as_str())
            .or_default()
            .push(job);
    }
    let empty_jobs: Vec<&model::WorkflowJobNode> = Vec::new();
    let mut reported: HashSet<String> = HashSet::new();

    for reference in references {
        let caller_path = topology_identifiers::workflow_path_from_id(&reference.consumer_job_id);
        let workflow_jobs = jobs_by_workflow.get(caller_path).unwrap_or(&empty_jobs);
        let job_lookup =
            CaseInsensitiveLookup::new(workflow_jobs.iter().map(|job| (job.key.clone(), *job)));
        let Resolution::Resolved {
            value: call_job, ..
        } = job_lookup.resolve(&reference.call_job_key)
        else {
            continue;
        };
        let call_job_id = call_job.id.clone();
        let Some(edge) = calls_by_job_id.get(call_job_id.as_str()) else {
            continue;
        };
        let Some(callee) =
            resolved_callable_callee(edge.local, edge.to.as_deref(), workflows_by_path)
        else {
            continue;
        };
        let Some(contract) = &callee.workflow_call else {
            continue;
        };
        let outputs = CaseInsensitiveLookup::new(
            contract
                .outputs
                .iter()
                .map(|(key, value)| (key.clone(), value)),
        );
        if !matches!(outputs.resolve(&reference.output), Resolution::Missing) {
            continue;
        }
        let key = format!(
            "{}|{}|{}",
            reference.consumer_job_id.to_lowercase(),
            call_job_id.to_lowercase(),
            reference.output.to_lowercase(),
        );
        if !reported.insert(key) {
            continue;
        }
        diagnostics.push(
            model::WorkflowTopologyDiagnostic::new(
                model::DiagnosticCode::UnknownWorkflowCallOutput,
                format!(
                    "{} references unknown output {} from {call_job_id} ({})",
                    reference.consumer_job_id, reference.output, callee.path
                ),
                caller_path,
            )
            .with_job(&reference.consumer_job_id)
            .with_call_job(&call_job_id)
            .with_callee(&callee.path),
        );
    }
}
