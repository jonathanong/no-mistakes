use super::merge_availability::MergeAvailability;
use super::evaluate::Index;
use super::LocatedFinding;
use super::{job_finding, workflow_finding as finding, Options};
use crate::codebase::workflow_topology::model::WorkflowTopology;

mod unlocked;

pub(super) fn lint(
    topology: &WorkflowTopology,
    index: &Index<'_>,
    opts: &Options,
    availability: &MergeAvailability<'_>,
) -> Vec<LocatedFinding> {
    let mut findings = Vec::new();
    findings.extend(edges(index, availability, &opts.required_direct_edges, true, true));
    findings.extend(edges(index, availability, &opts.forbidden_direct_edges, true, false));
    findings.extend(edges(index, availability, &opts.required_transitive_edges, false, true));
    findings.extend(edges(index, availability, &opts.forbidden_transitive_edges, false, false));
    findings.extend(fan_ins(index, opts, availability));
    findings.extend(callers(index, opts, availability));
    findings.extend(unlocked::unlocked(topology, opts, availability));
    findings
}

fn edges(
    index: &Index<'_>,
    availability: &MergeAvailability<'_>,
    rules: &[[String; 2]],
    direct: bool,
    required: bool,
) -> Vec<LocatedFinding> {
    let kind = if direct { "direct" } else { "transitive" };
    rules
        .iter()
        .filter_map(|[from, to]| {
            let absence_known = if direct {
                availability.direct_edge_known(from, to)
            } else {
                availability.transitive_edges_known(from) && availability.transitive_edges_known(to)
            };
            if required && !absence_known {
                return None;
            }
            if !index.jobs.contains_key(from.as_str()) || !index.jobs.contains_key(to.as_str()) {
                return required.then(|| {
                    job_finding(to, format!("required {kind} edge missing: {from} -> {to}"))
                });
            }
            let downstream = if direct {
                index.direct_downstream(from)
            } else {
                index.transitive_downstream(from)
            };
            let present = downstream.contains(&to.as_str());
            (present != required).then(|| {
                job_finding(
                    to,
                    format!(
                        "{} {kind} edge {}: {from} -> {to}",
                        if required { "required" } else { "forbidden" },
                        if required { "missing" } else { "present" }
                    ),
                )
            })
        })
        .collect()
}

fn fan_ins(index: &Index<'_>, opts: &Options, availability: &MergeAvailability<'_>) -> Vec<LocatedFinding> {
    opts.exact_fan_ins
        .iter()
        .filter(|(job, _)| availability.needs_known(job))
        .map(|(job_id, expected)| {
            if !index.jobs.contains_key(job_id.as_str()) {
                return job_finding(job_id, format!("exact fan-in target missing: {job_id}"));
            }
            let actual = index.direct_upstream(job_id);
            let mut expected: Vec<&str> = expected.iter().map(String::as_str).collect();
            expected.sort_unstable();
            if actual == expected {
                return job_finding(job_id, String::new());
            }
            job_finding(
                job_id,
                format!(
                    "exact fan-in mismatch: {job_id}: expected {}, got {}",
                    expected.join(", "),
                    actual.join(", ")
                ),
            )
        })
        .filter(|finding| !finding.finding.message.is_empty())
        .collect()
}

fn callers(index: &Index<'_>, opts: &Options, availability: &MergeAvailability<'_>) -> Vec<LocatedFinding> {
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
            findings.push(finding(path, format!("caller allowlist missing: {path}")));
        }
    }
    for path in opts.exact_caller_jobs.keys() {
        if !callable.contains(&path.as_str()) && availability.callable_known(path) {
            findings.push(finding(path, format!("caller allowlist stale: {path}")));
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
        let unexpected = actual.iter().any(|job| expected.binary_search(job).is_err());
        let missing = expected.iter().any(|job| {
            actual.binary_search(job).is_err()
                && availability.caller_known(job, index.jobs.contains_key(job))
        });
        if unexpected || missing {
            findings.push(finding(
                path,
                format!(
                    "caller allowlist mismatch: {path}: expected {}, got {}",
                    expected.join(", "),
                    actual.join(", ")
                ),
            ));
        }
    }
    findings
}
