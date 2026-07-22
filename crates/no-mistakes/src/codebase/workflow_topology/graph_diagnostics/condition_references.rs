//! `if:` condition reference diagnostics (`needs.*`, `steps.*`, duplicate
//! step ids), split out of [`super`] to stay under the crate's per-file
//! line limit.

use super::super::case_insensitive_lookup::{CaseInsensitiveLookup, Resolution};
use super::super::expression_references;
use super::super::model;
use super::super::topology_identifiers;
use std::collections::{BTreeSet, HashMap, HashSet};

pub(super) fn diagnose_condition_references(
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
