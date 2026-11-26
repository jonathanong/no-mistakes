use super::merge_availability::MergeAvailability;
use super::evaluate::Index;
use super::LocatedFinding;
use super::{job_finding as finding, Options, StepSelector};
use crate::codebase::workflow_topology::artifact_types::ArtifactDeclaration;
use crate::codebase::workflow_topology::model::{
    WorkflowStep, WorkflowTopology, WorkflowTopologyEdge,
};

pub(super) fn lint(
    topology: &WorkflowTopology,
    index: &Index<'_>,
    opts: &Options,
    availability: &MergeAvailability<'_>,
) -> Vec<LocatedFinding> {
    let mut findings = step_orders(index, opts, availability);
    findings.extend(artifact_edges(topology, index, opts, availability));
    findings
}

fn artifact_edges(
    topology: &WorkflowTopology,
    index: &Index<'_>,
    opts: &Options,
    availability: &MergeAvailability<'_>,
) -> Vec<LocatedFinding> {
    let artifacts_known = availability.artifacts_known();
    opts.required_artifact_edges
        .iter()
        .filter_map(|rule| {
            let present = topology.edges.iter().any(|edge| match edge {
                WorkflowTopologyEdge::Artifact(artifact) => {
                    artifact.from == rule.from
                        && artifact.to == rule.to
                        && artifact.name == rule.name
                        && rule
                            .match_kind
                            .as_deref()
                            .is_none_or(|kind| artifact.match_kind.as_str() == kind)
                }
                _ => false,
            });
            if present {
                return None;
            }
            if !artifacts_known
                && !artifact_endpoint_missing(index, availability, &rule.from, true)
                && !artifact_endpoint_missing(index, availability, &rule.to, false)
            {
                return None;
            }
            let label = match &rule.match_kind {
                Some(kind) => format!("{} -> {}: {} [{kind}]", rule.from, rule.to, rule.name),
                None => format!("{} -> {}: {}", rule.from, rule.to, rule.name),
            };
            Some(finding(&rule.to, format!("required artifact edge missing: {label}")))
        })
        .collect()
}

fn artifact_endpoint_missing(
    index: &Index<'_>,
    availability: &MergeAvailability<'_>,
    job: &str,
    upload: bool,
) -> bool {
    let Some(job) = index.jobs.get(job) else {
        return availability.job_membership_known(job);
    };
    availability.artifact_steps_known(&job.id)
        && job.steps.iter().all(|step| {
            !matches!((&step.artifact, upload),
                (Some(ArtifactDeclaration::Upload(_)), true)
                    | (Some(ArtifactDeclaration::Download(_)), false))
        })
}

fn step_orders(index: &Index<'_>, opts: &Options, availability: &MergeAvailability<'_>) -> Vec<LocatedFinding> {
    opts.step_orders
        .iter()
        .filter(|rule| availability.steps_known(&rule.job_id, index.jobs.contains_key(rule.job_id.as_str())))
        .filter_map(|rule| {
            let Some(job) = index.jobs.get(rule.job_id.as_str()) else {
                return Some(finding(
                    &rule.job_id,
                    format!("step-order job missing: {}", rule.job_id),
                ));
            };
            let mut prior = -1i32;
            for selector in &rule.steps {
                let Some(step) = job
                    .steps
                    .iter()
                    .find(|candidate| matches_selector(selector, candidate))
                else {
                    let label = selector
                        .id
                        .as_deref()
                        .or(selector.uses.as_deref())
                        .or(selector.name.as_deref())
                        .unwrap_or("<step>");
                    return Some(finding(
                        &rule.job_id,
                        format!("required ordered step missing: {}: {label}", rule.job_id),
                    ));
                };
                if (step.index as i32) <= prior {
                    let label = selector
                        .id
                        .as_deref()
                        .or(selector.uses.as_deref())
                        .or(selector.name.as_deref())
                        .unwrap_or("<step>");
                    return Some(finding(
                        &rule.job_id,
                        format!("required step order invalid: {}: {label}", rule.job_id),
                    ));
                }
                prior = step.index as i32;
            }
            None
        })
        .collect()
}

fn matches_selector(selector: &StepSelector, step: &WorkflowStep) -> bool {
    selector
        .id
        .as_deref()
        .is_none_or(|id| step.id.as_deref() == Some(id))
        && selector
            .uses
            .as_deref()
            .is_none_or(|uses| step.uses.as_deref() == Some(uses))
        && selector
            .name
            .as_deref()
            .is_none_or(|name| step.name.as_deref() == Some(name))
}
