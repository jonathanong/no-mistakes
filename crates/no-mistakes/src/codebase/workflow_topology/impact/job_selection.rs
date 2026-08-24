use super::super::model::{WorkflowTopology, WorkflowTopologyEdge};
use super::yaml::yaml_at;
use git2::{Repository, Tree};
use std::collections::{BTreeSet, HashMap};

pub(super) fn entry_changed_jobs(
    repo: &Repository,
    base: &Tree<'_>,
    head: &Tree<'_>,
    entry: &str,
) -> BTreeSet<String> {
    let (Some(base), Some(head)) = (yaml_at(repo, base, entry), yaml_at(repo, head, entry)) else {
        return BTreeSet::new();
    };
    let jobs_key = serde_yaml::Value::String("jobs".into());
    let (Some(base_jobs), Some(head_jobs)) = (
        base.as_mapping()
            .and_then(|map| map.get(&jobs_key))
            .and_then(serde_yaml::Value::as_mapping),
        head.as_mapping()
            .and_then(|map| map.get(&jobs_key))
            .and_then(serde_yaml::Value::as_mapping),
    ) else {
        return BTreeSet::new();
    };
    base_jobs
        .keys()
        .chain(head_jobs.keys())
        .filter_map(serde_yaml::Value::as_str)
        .filter(|key| {
            base_jobs.get(serde_yaml::Value::String((**key).into()))
                != head_jobs.get(serde_yaml::Value::String((**key).into()))
        })
        .map(str::to_owned)
        .collect()
}

pub(super) fn entry_job_ids(
    entry: &str,
    changed: &BTreeSet<String>,
    base: &WorkflowTopology,
    head: &WorkflowTopology,
) -> BTreeSet<String> {
    base.jobs
        .iter()
        .chain(&head.jobs)
        .filter(|job| job.workflow_id == entry && changed.contains(&job.key))
        .map(|job| job.id.clone())
        .collect()
}

pub(super) fn add_needs_closure(
    selected: &mut BTreeSet<String>,
    base: &WorkflowTopology,
    head: &WorkflowTopology,
) {
    let mut prerequisites = HashMap::<String, Vec<String>>::new();
    let mut dependents = HashMap::<String, Vec<String>>::new();
    for topology in [base, head] {
        for edge in &topology.edges {
            if let WorkflowTopologyEdge::Needs(edge) = edge {
                prerequisites
                    .entry(edge.to.clone())
                    .or_default()
                    .push(edge.from.clone());
                dependents
                    .entry(edge.from.clone())
                    .or_default()
                    .push(edge.to.clone());
            }
        }
    }
    let changed = selected.clone();
    let mut pending: Vec<String> = changed.iter().cloned().collect();
    while let Some(job) = pending.pop() {
        for dependent in dependents.remove(&job).unwrap_or_default() {
            if selected.insert(dependent.clone()) {
                pending.push(dependent);
            }
        }
    }
    // A downstream job can introduce its own prerequisite. Expand upstream
    // only after the affected dependent closure is complete, while never
    // revisiting dependents of those prerequisites.
    let mut pending: Vec<String> = selected.iter().cloned().collect();
    while let Some(job) = pending.pop() {
        for prerequisite in prerequisites.remove(&job).unwrap_or_default() {
            if selected.insert(prerequisite.clone()) {
                pending.push(prerequisite);
            }
        }
    }
}

pub(super) fn entry_change_is_global(
    repo: &Repository,
    base: &Tree<'_>,
    head: &Tree<'_>,
    entry: &str,
) -> bool {
    let (Some(base), Some(head)) = (yaml_at(repo, base, entry), yaml_at(repo, head, entry)) else {
        return true;
    };
    let (Some(base), Some(head)) = (base.as_mapping(), head.as_mapping()) else {
        return true;
    };
    let jobs = serde_yaml::Value::String("jobs".into());
    // A missing or non-mapping `jobs` value cannot be projected job-by-job.
    // Treating it as an empty mapping would silently return bounded-empty.
    if base
        .get(&jobs)
        .and_then(serde_yaml::Value::as_mapping)
        .is_none()
        || head
            .get(&jobs)
            .and_then(serde_yaml::Value::as_mapping)
            .is_none()
    {
        return true;
    }
    let mut base = base.clone();
    let mut head = head.clone();
    base.remove(&jobs);
    head.remove(&jobs);
    base != head
}
