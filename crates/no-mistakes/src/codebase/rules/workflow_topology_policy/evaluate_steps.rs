use super::evaluate::Index;
use super::{finding, Options, StepSelector};
use crate::codebase::rules::RuleFinding;
use crate::codebase::workflow_topology::model::{
    WorkflowStep, WorkflowTopology, WorkflowTopologyEdge,
};

pub(super) fn lint(
    topology: &WorkflowTopology,
    index: &Index<'_>,
    opts: &Options,
) -> Vec<RuleFinding> {
    let mut findings = artifact_edges(topology, index, opts);
    findings.extend(step_orders(index, opts));
    findings
}

fn artifact_edges(
    topology: &WorkflowTopology,
    index: &Index<'_>,
    opts: &Options,
) -> Vec<RuleFinding> {
    opts.required_artifact_edges
        .iter()
        .map(|rule| {
            let label = match &rule.match_kind {
                Some(kind) => format!("{} -> {}: {} [{kind}]", rule.from, rule.to, rule.name),
                None => format!("{} -> {}: {}", rule.from, rule.to, rule.name),
            };
            if !index.jobs.contains_key(rule.from.as_str())
                || !index.jobs.contains_key(rule.to.as_str())
            {
                return finding(format!("required artifact edge missing: {label}"));
            }
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
                finding(String::new())
            } else {
                finding(format!("required artifact edge missing: {label}"))
            }
        })
        .filter(|finding| !finding.message.is_empty())
        .collect()
}

fn step_orders(index: &Index<'_>, opts: &Options) -> Vec<RuleFinding> {
    opts.step_orders
        .iter()
        .filter_map(|rule| {
            let Some(job) = index.jobs.get(rule.job_id.as_str()) else {
                return Some(finding(format!("step-order job missing: {}", rule.job_id)));
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
                    return Some(finding(format!(
                        "required ordered step missing: {}: {label}",
                        rule.job_id
                    )));
                };
                if (step.index as i32) <= prior {
                    let label = selector
                        .id
                        .as_deref()
                        .or(selector.uses.as_deref())
                        .or(selector.name.as_deref())
                        .unwrap_or("<step>");
                    return Some(finding(format!(
                        "required step order invalid: {}: {label}",
                        rule.job_id
                    )));
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
