//! Job-graph diagnostics (needs cycles, condition references, duplicate
//! step ids, duplicate workflow names), ported from `graph-diagnostics.mts`.

use super::case_insensitive_lookup::{CaseInsensitiveLookup, Resolution};
use super::expression_references;
use super::graph_algorithms;
use super::model;
use super::topology_identifiers;
use std::collections::{BTreeSet, HashMap, HashSet};

pub fn diagnose_job_graph(
    jobs: &[model::WorkflowJobNode],
    edges: &[model::WorkflowTopologyEdge],
    diagnostics: &mut Vec<model::WorkflowTopologyDiagnostic>,
) {
    let jobs_by_id: HashSet<&str> = jobs.iter().map(|job| job.id.as_str()).collect();
    let mut needs_by_job: HashMap<String, HashSet<String>> = jobs
        .iter()
        .map(|job| (job.id.clone(), HashSet::new()))
        .collect();
    for edge in edges {
        let model::WorkflowTopologyEdge::Needs(needs) = edge else {
            continue;
        };
        if !jobs_by_id.contains(needs.from.as_str()) || !jobs_by_id.contains(needs.to.as_str()) {
            continue;
        }
        needs_by_job
            .get_mut(&needs.to)
            .expect("needs.to is a known job id")
            .insert(needs.from.clone());
    }
    diagnose_job_cycles(&needs_by_job, diagnostics);

    let mut jobs_by_workflow: HashMap<&str, Vec<&model::WorkflowJobNode>> = HashMap::new();
    for job in jobs {
        jobs_by_workflow
            .entry(job.workflow_id.as_str())
            .or_default()
            .push(job);
    }
    let job_lookups: HashMap<&str, CaseInsensitiveLookup<&model::WorkflowJobNode>> =
        jobs_by_workflow
            .into_iter()
            .map(|(path, workflow_jobs)| {
                let entries = workflow_jobs.into_iter().map(|job| (job.key.clone(), job));
                (path, CaseInsensitiveLookup::new(entries))
            })
            .collect();

    let empty_needs: HashSet<String> = HashSet::new();
    for job in jobs {
        let lookup = &job_lookups[job.workflow_id.as_str()];
        let direct_needs = needs_by_job.get(&job.id).unwrap_or(&empty_needs);
        diagnose_condition_references(job, direct_needs, lookup, diagnostics);
    }
}

pub fn diagnose_duplicate_workflow_names(
    workflows: &[model::WorkflowNode],
    diagnostics: &mut Vec<model::WorkflowTopologyDiagnostic>,
) {
    let mut by_name: HashMap<String, Vec<&model::WorkflowNode>> = HashMap::new();
    for workflow in workflows {
        by_name
            .entry(workflow.name.to_lowercase())
            .or_default()
            .push(workflow);
    }
    for conflicts in by_name.values() {
        if conflicts.len() < 2 {
            continue;
        }
        let mut paths: Vec<&str> = conflicts
            .iter()
            .map(|workflow| workflow.path.as_str())
            .collect();
        paths.sort();
        for workflow in conflicts {
            let quoted_name = serde_json::to_string(&workflow.name).unwrap_or_default();
            diagnostics.push(model::WorkflowTopologyDiagnostic::new(
                model::DiagnosticCode::DuplicateWorkflowName,
                format!(
                    "workflow name {quoted_name} conflicts across {}",
                    paths.join(", ")
                ),
                workflow.path.clone(),
            ));
        }
    }
}

fn diagnose_job_cycles(
    needs_by_job: &HashMap<String, HashSet<String>>,
    diagnostics: &mut Vec<model::WorkflowTopologyDiagnostic>,
) {
    for component in graph_algorithms::strongly_connected_components(needs_by_job) {
        let is_self_cycle = component.len() == 1
            && needs_by_job
                .get(&component[0])
                .is_some_and(|set| set.contains(&component[0]));
        if component.len() < 2 && !is_self_cycle {
            continue;
        }
        let mut sorted = component;
        sorted.sort();
        diagnostics.push(
            model::WorkflowTopologyDiagnostic::new(
                model::DiagnosticCode::JobDependencyCycle,
                format!("job dependency cycle: {}", sorted.join(", ")),
                topology_identifiers::workflow_path_from_id(&sorted[0]),
            )
            .with_job(&sorted[0]),
        );
    }
}

fn diagnose_condition_references(
    job: &model::WorkflowJobNode,
    direct_needs: &HashSet<String>,
    job_lookup: &CaseInsensitiveLookup<&model::WorkflowJobNode>,
    diagnostics: &mut Vec<model::WorkflowTopologyDiagnostic>,
) {
    let direct_need_keys: HashSet<String> = direct_needs
        .iter()
        .map(|id| {
            id.rsplit_once('#')
                .map_or(id.as_str(), |(_, key)| key)
                .to_string()
        })
        .collect();

    let mut referenced_needs: BTreeSet<String> =
        expression_references::static_references(job.condition.as_deref(), "needs")
            .into_iter()
            .collect();
    for step in &job.steps {
        referenced_needs.extend(expression_references::static_references(
            step.condition.as_deref(),
            "needs",
        ));
    }

    for reference in &referenced_needs {
        match job_lookup.resolve(reference) {
            Resolution::Ambiguous => continue,
            Resolution::Resolved { key, .. } if direct_need_keys.contains(key) => continue,
            _ => {}
        }
        diagnostics.push(
            model::WorkflowTopologyDiagnostic::new(
                model::DiagnosticCode::MissingNeedsDependency,
                format!(
                    "{} references needs.{reference} without a direct dependency",
                    job.id
                ),
                topology_identifiers::workflow_path_from_id(&job.id),
            )
            .with_job(&job.id),
        );
    }

    let mut step_indexes: HashMap<String, Vec<u32>> = HashMap::new();
    for step in &job.steps {
        if let Some(id) = &step.id {
            step_indexes.entry(id.clone()).or_default().push(step.index);
        }
    }
    for (step_id, indexes) in &step_indexes {
        if indexes.len() < 2 {
            continue;
        }
        diagnostics.push(
            model::WorkflowTopologyDiagnostic::new(
                model::DiagnosticCode::DuplicateStepId,
                format!("{} declares duplicate step id {step_id}", job.id),
                topology_identifiers::workflow_path_from_id(&job.id),
            )
            .with_job(&job.id),
        );
    }

    let mut reported_step_references: HashSet<String> = HashSet::new();
    diagnose_step_references(
        job,
        job.condition.as_deref(),
        -1,
        &step_indexes,
        &mut reported_step_references,
        diagnostics,
    );
    for step in &job.steps {
        diagnose_step_references(
            job,
            step.condition.as_deref(),
            i64::from(step.index),
            &step_indexes,
            &mut reported_step_references,
            diagnostics,
        );
    }
}

/// `current_index` is `-1` for the job-level `if:` (nothing has executed
/// yet) and the step's own index for a step-level `if:`.
#[allow(clippy::too_many_arguments)]
fn diagnose_step_references(
    job: &model::WorkflowJobNode,
    condition: Option<&str>,
    current_index: i64,
    step_indexes: &HashMap<String, Vec<u32>>,
    reported: &mut HashSet<String>,
    diagnostics: &mut Vec<model::WorkflowTopologyDiagnostic>,
) {
    for reference in expression_references::static_references(condition, "steps") {
        match step_indexes.get(&reference) {
            None if reported.insert(format!("unknown-step-reference|{reference}")) => {
                diagnostics.push(
                    model::WorkflowTopologyDiagnostic::new(
                        model::DiagnosticCode::UnknownStepReference,
                        format!("{} references unknown step {reference}", job.id),
                        topology_identifiers::workflow_path_from_id(&job.id),
                    )
                    .with_job(&job.id),
                );
            }
            Some(indexes)
                if !indexes
                    .iter()
                    .any(|index| i64::from(*index) < current_index)
                    && reported.insert(format!("non-prior-step-reference|{reference}")) =>
            {
                diagnostics.push(
                    model::WorkflowTopologyDiagnostic::new(
                        model::DiagnosticCode::NonPriorStepReference,
                        format!(
                            "{} references step {reference} before it has executed",
                            job.id
                        ),
                        topology_identifiers::workflow_path_from_id(&job.id),
                    )
                    .with_job(&job.id),
                );
            }
            _ => {}
        }
    }
}
