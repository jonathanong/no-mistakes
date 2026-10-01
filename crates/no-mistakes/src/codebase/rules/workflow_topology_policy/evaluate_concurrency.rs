use super::concurrency_compile::CompiledIntent;
use super::concurrency_scope::actual_scope;
use super::finding;
use crate::codebase::rules::RuleFinding;
use crate::codebase::workflow_topology::model::{
    ConcurrencyValue, WorkflowConcurrency, WorkflowTopology,
};
use regex::Regex;
use std::collections::BTreeMap;
use std::sync::OnceLock;

pub(super) fn lint(
    topology: &WorkflowTopology,
    policy: &BTreeMap<String, CompiledIntent>,
) -> Vec<RuleFinding> {
    if policy.is_empty() {
        return Vec::new();
    }
    let owners = owners(topology);
    let mut findings = Vec::new();
    for id in owners.keys() {
        if !policy.contains_key(*id) {
            findings.push(finding(format!("concurrency intent missing: {id}")));
        }
    }
    for id in policy.keys() {
        if !owners.contains_key(id.as_str()) {
            findings.push(finding(format!("concurrency intent stale: {id}")));
        }
    }
    for (id, intent) in policy {
        let Some(concurrency) = owners.get(id.as_str()) else {
            continue;
        };
        findings.extend(compare(id, concurrency, intent));
    }
    findings
}

fn owners(topology: &WorkflowTopology) -> BTreeMap<&str, &WorkflowConcurrency> {
    let mut owners = BTreeMap::new();
    for workflow in &topology.workflows {
        if let Some(concurrency) = &workflow.concurrency {
            owners.insert(workflow.path.as_str(), concurrency);
        }
    }
    for job in &topology.jobs {
        if let Some(concurrency) = &job.concurrency {
            owners.insert(job.id.as_str(), concurrency);
        }
    }
    owners
}

fn compare(
    id: &str,
    concurrency: &WorkflowConcurrency,
    intent: &CompiledIntent,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    let pending = if concurrency.effective.queue == "max" {
        "fifo"
    } else {
        "coalesce-latest"
    };
    if pending != intent.pending {
        findings.push(finding(format!(
            "concurrency pending mismatch: {id}: expected {}, got {pending}",
            intent.pending
        )));
    }
    let cancellation = match &concurrency.effective.cancel_in_progress {
        ConcurrencyValue::Text(value) => {
            if !complete_expression(value) {
                findings.push(finding(format!(
                    "conditional cancel-in-progress expression invalid: {id}: {value}"
                )));
            }
            "conditional"
        }
        ConcurrencyValue::Bool(true) => "cancel-running",
        ConcurrencyValue::Bool(false) => "retain-running",
    };
    if cancellation != intent.cancellation {
        findings.push(finding(format!(
            "concurrency cancellation mismatch: {id}: expected {}, got {cancellation}",
            intent.cancellation
        )));
    }
    let actual = actual_scope(&concurrency.effective.group);
    if actual != intent.scope {
        findings.push(finding(format!(
            "concurrency scope mismatch: {id}: expected {}, got {}",
            intent.scope.join(", "),
            actual.join(", ")
        )));
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
