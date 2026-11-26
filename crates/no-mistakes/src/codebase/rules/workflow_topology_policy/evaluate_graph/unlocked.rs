use super::{finding, LocatedFinding, MergeAvailability, Options, WorkflowTopology};

pub(super) fn unlocked(topology: &WorkflowTopology, opts: &Options, availability: &MergeAvailability<'_>) -> Vec<LocatedFinding> {
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
        let has_unlocked_job = availability.concurrency_known(&workflow.path, true)
            && workflow.concurrency.is_none()
            && workflow.job_ids.iter().any(|id| {
                availability.concurrency_known(id, true)
                    && jobs.get(id.as_str()).is_some_and(|job| job.concurrency.is_none())
            });
        let lock_coverage_known = availability.concurrency_known(&workflow.path, true)
            && (workflow.concurrency.is_some()
                || (availability.inventory_known(&workflow.path)
                    && workflow.job_ids.iter().all(|id| availability.concurrency_known(id, true))));
        if has_unlocked_job && reason.is_none() {
            findings.push(finding(
                &workflow.path,
                format!("lock intent missing: {}", workflow.path),
            ));
        }
        if !has_unlocked_job && lock_coverage_known && reason.is_some() {
            findings.push(finding(
                &workflow.path,
                format!("unlocked reason stale: {}", workflow.path),
            ));
        }
        if reason.is_some_and(|value| value.trim().is_empty()) {
            findings.push(finding(
                &workflow.path,
                format!("unlocked reason empty: {}", workflow.path),
            ));
        }
    }
    for path in opts.unlocked_workflow_reasons.keys() {
        if !topology
            .workflows
            .iter()
            .any(|workflow| workflow.path == *path)
        {
            findings.push(finding(path, format!("unlocked reason stale: {path}")));
        }
    }
    findings
}
