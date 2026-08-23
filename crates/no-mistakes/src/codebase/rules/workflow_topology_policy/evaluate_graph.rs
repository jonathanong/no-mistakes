use super::evaluate::Index;
use super::{finding, Options};
use crate::codebase::rules::RuleFinding;
use crate::codebase::workflow_topology::model::WorkflowTopology;

pub(super) fn lint(
    topology: &WorkflowTopology,
    index: &Index<'_>,
    opts: &Options,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    findings.extend(edges(index, &opts.required_direct_edges, true, true));
    findings.extend(edges(index, &opts.forbidden_direct_edges, true, false));
    findings.extend(edges(index, &opts.required_transitive_edges, false, true));
    findings.extend(edges(index, &opts.forbidden_transitive_edges, false, false));
    findings.extend(fan_ins(index, opts));
    findings.extend(callers(index, opts));
    findings.extend(unlocked(topology, opts));
    findings
}

fn edges(
    index: &Index<'_>,
    rules: &[[String; 2]],
    direct: bool,
    required: bool,
) -> Vec<RuleFinding> {
    let kind = if direct { "direct" } else { "transitive" };
    rules
        .iter()
        .filter_map(|[from, to]| {
            if !index.jobs.contains_key(from.as_str()) || !index.jobs.contains_key(to.as_str()) {
                return required
                    .then(|| finding(format!("required {kind} edge missing: {from} -> {to}")));
            }
            let downstream = if direct {
                index.direct_downstream(from)
            } else {
                index.transitive_downstream(from)
            };
            let present = downstream.contains(&to.as_str());
            (present != required).then(|| {
                finding(format!(
                    "{} {kind} edge {}: {from} -> {to}",
                    if required { "required" } else { "forbidden" },
                    if required { "missing" } else { "present" }
                ))
            })
        })
        .collect()
}

fn fan_ins(index: &Index<'_>, opts: &Options) -> Vec<RuleFinding> {
    opts.exact_fan_ins
        .iter()
        .map(|(job_id, expected)| {
            if !index.jobs.contains_key(job_id.as_str()) {
                return finding(format!("exact fan-in target missing: {job_id}"));
            }
            let actual = index.direct_upstream(job_id);
            let mut expected: Vec<&str> = expected.iter().map(String::as_str).collect();
            expected.sort_unstable();
            if actual == expected {
                return finding(String::new());
            }
            finding(format!(
                "exact fan-in mismatch: {job_id}: expected {}, got {}",
                expected.join(", "),
                actual.join(", ")
            ))
        })
        .filter(|finding| !finding.message.is_empty())
        .collect()
}

fn callers(index: &Index<'_>, opts: &Options) -> Vec<RuleFinding> {
    if opts.exact_caller_jobs.is_empty() {
        return Vec::new();
    }
    let callable: Vec<&str> = index
        .workflows
        .values()
        .filter(|workflow| workflow.callable)
        .map(|workflow| workflow.path.as_str())
        .collect();
    let mut findings = Vec::new();
    for path in &callable {
        if !opts.exact_caller_jobs.contains_key(*path) {
            findings.push(finding(format!("caller allowlist missing: {path}")));
        }
    }
    for path in opts.exact_caller_jobs.keys() {
        if !callable.contains(&path.as_str()) {
            findings.push(finding(format!("caller allowlist stale: {path}")));
        }
    }
    for (path, expected) in &opts.exact_caller_jobs {
        if !index
            .workflows
            .get(path.as_str())
            .is_some_and(|workflow| workflow.callable)
        {
            continue;
        }
        let actual = index.direct_caller_jobs(path);
        let mut expected: Vec<&str> = expected.iter().map(String::as_str).collect();
        expected.sort_unstable();
        if actual != expected {
            findings.push(finding(format!(
                "caller allowlist mismatch: {path}: expected {}, got {}",
                expected.join(", "),
                actual.join(", ")
            )));
        }
    }
    findings
}

fn unlocked(topology: &WorkflowTopology, opts: &Options) -> Vec<RuleFinding> {
    if opts.unlocked_workflow_reasons.is_empty() {
        return Vec::new();
    }
    let mut findings = Vec::new();
    let jobs: std::collections::HashMap<&str, _> = topology
        .jobs
        .iter()
        .map(|job| (job.id.as_str(), job))
        .collect();
    for workflow in &topology.workflows {
        let reason = opts.unlocked_workflow_reasons.get(&workflow.path);
        let has_unlocked_job = workflow.concurrency.is_none()
            && workflow.job_ids.iter().any(|id| {
                jobs.get(id.as_str())
                    .is_some_and(|job| job.concurrency.is_none())
            });
        if has_unlocked_job && reason.is_none() {
            findings.push(finding(format!("lock intent missing: {}", workflow.path)));
        }
        if !has_unlocked_job && reason.is_some() {
            findings.push(finding(format!("unlocked reason stale: {}", workflow.path)));
        }
        if reason.is_some_and(|value| value.trim().is_empty()) {
            findings.push(finding(format!("unlocked reason empty: {}", workflow.path)));
        }
    }
    for path in opts.unlocked_workflow_reasons.keys() {
        if !topology
            .workflows
            .iter()
            .any(|workflow| workflow.path == *path)
        {
            findings.push(finding(format!("unlocked reason stale: {path}")));
        }
    }
    findings
}
