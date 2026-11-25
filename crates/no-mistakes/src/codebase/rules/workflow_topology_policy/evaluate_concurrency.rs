use super::merge_availability::MergeAvailability;
use super::concurrency_compile::CompiledIntent;
use super::concurrency_scope::actual_scope;
use super::{finding, workflow_finding, FindingOwner, LocatedFinding};
use crate::codebase::workflow_topology::model::{
    ConcurrencyValue, WorkflowConcurrency, WorkflowTopology,
};
use crate::config::source_locations::ConfigPath;
use rayon::prelude::*;
use regex::Regex;
use std::collections::BTreeMap;
use std::sync::OnceLock;

pub(super) fn lint_reusable_workflows(topology: &WorkflowTopology) -> Vec<LocatedFinding> {
    topology
        .workflows
        .par_iter()
        .filter(|workflow| {
            workflow.concurrency.is_some()
                && workflow
                    .triggers
                    .iter()
                    .any(|trigger| trigger.event == "workflow_call")
        })
        .map(|workflow| {
            let mut finding = workflow_finding(
                &workflow.path,
                "forbidden workflow-level concurrency on a reusable workflow. Declare concurrency on the caller or on individual jobs.".to_string(),
            );
            finding.path = ConfigPath::default().key("concurrency");
            finding
        })
        .collect()
}

pub(super) fn lint(
    topology: &WorkflowTopology,
    policy: &BTreeMap<String, CompiledIntent>,
    availability: &MergeAvailability<'_>,
) -> Vec<LocatedFinding> {
    if policy.is_empty() {
        return Vec::new();
    }
    let owners = owners(topology);
    let mut findings = Vec::new();
    for (id, (concurrency, owner)) in &owners {
        if concurrency.is_some() && !policy.contains_key(*id) {
            findings.push(finding(*owner, format!("concurrency intent missing: {id}")));
        }
    }
    for id in policy.keys() {
        if !availability.concurrency_known(id, owners.contains_key(id.as_str())) {
            continue;
        }
        if owners
            .get(id.as_str())
            .is_none_or(|(concurrency, _)| concurrency.is_none())
        {
            let owner = owners
                .get(id.as_str())
                .map(|(_, owner)| *owner)
                .unwrap_or_else(|| match id.rsplit_once('#') {
                    Some((file, _)) if owners.contains_key(file) => FindingOwner::Job(id),
                    _ => FindingOwner::Workflow(id),
                });
            findings.push(finding(owner, format!("concurrency intent stale: {id}")));
        }
    }
    for (id, intent) in policy {
        let Some((Some(concurrency), owner)) = owners.get(id.as_str()) else {
            continue;
        };
        if availability.concurrency_known(id, true) {
            findings.extend(compare(id, *owner, concurrency, intent));
        }
    }
    findings
}

fn owners(
    topology: &WorkflowTopology,
) -> BTreeMap<&str, (Option<&WorkflowConcurrency>, FindingOwner<'_>)> {
    let mut owners = BTreeMap::new();
    for workflow in &topology.workflows {
        owners.insert(
            workflow.path.as_str(),
            (
                workflow.concurrency.as_ref(),
                FindingOwner::Workflow(&workflow.path),
            ),
        );
    }
    for job in &topology.jobs {
        owners.insert(
            job.id.as_str(),
            (job.concurrency.as_ref(), FindingOwner::Job(&job.id)),
        );
    }
    owners
}

fn compare(
    id: &str,
    owner: FindingOwner<'_>,
    concurrency: &WorkflowConcurrency,
    intent: &CompiledIntent,
) -> Vec<LocatedFinding> {
    let mut findings = Vec::new();
    let pending = if concurrency.effective.queue == "max" {
        "fifo"
    } else {
        "coalesce-latest"
    };
    if pending != intent.pending {
        findings.push(finding(
            owner,
            format!(
                "concurrency pending mismatch: {id}: expected {}, got {pending}",
                intent.pending
            ),
        ));
    }
    let cancellation = match &concurrency.effective.cancel_in_progress {
        ConcurrencyValue::Text(value) => {
            if !complete_expression(value) {
                findings.push(finding(
                    owner,
                    format!("conditional cancel-in-progress expression invalid: {id}: {value}"),
                ));
            }
            "conditional"
        }
        ConcurrencyValue::Bool(true) => "cancel-running",
        ConcurrencyValue::Bool(false) => "retain-running",
    };
    if cancellation != intent.cancellation {
        findings.push(finding(
            owner,
            format!(
                "concurrency cancellation mismatch: {id}: expected {}, got {cancellation}",
                intent.cancellation
            ),
        ));
    }
    let actual = actual_scope(&concurrency.effective.group);
    if actual != intent.scope {
        findings.push(finding(
            owner,
            format!(
                "concurrency scope mismatch: {id}: expected {}, got {}",
                intent.scope.join(", "),
                actual.join(", ")
            ),
        ));
    }
    findings
}

fn complete_expression(value: &str) -> bool {
    expression_pattern().is_match(value.trim())
}

fn expression_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r"(?s)^\$\{\{\s*\S.*\}\}$").expect("cancel-in-progress expression pattern")
    })
}
