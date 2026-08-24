use super::super::model::{WorkflowTopology, WorkflowTopologyEdge};
use std::collections::{BTreeSet, HashMap};

pub(super) fn affected_workflow_closure(
    changed: &BTreeSet<String>,
    base: &WorkflowTopology,
    head: &WorkflowTopology,
) -> BTreeSet<String> {
    let mut callers = HashMap::<String, Vec<String>>::new();
    let mut owners = HashMap::<String, String>::new();
    for topology in [base, head] {
        for job in &topology.jobs {
            owners.insert(job.id.clone(), job.workflow_id.clone());
        }
        for edge in &topology.edges {
            if let WorkflowTopologyEdge::Calls(call) = edge {
                if let (Some(callee), Some(caller)) = (&call.to, owners.get(&call.from)) {
                    callers
                        .entry(callee.clone())
                        .or_default()
                        .push(caller.clone());
                }
            }
        }
    }
    let mut affected = changed.clone();
    let mut pending: Vec<String> = changed.iter().cloned().collect();
    while let Some(workflow) = pending.pop() {
        for caller in callers.remove(&workflow).unwrap_or_default() {
            if affected.insert(caller.clone()) {
                pending.push(caller);
            }
        }
    }
    affected
}

pub(super) fn root_callers(
    entry: &str,
    changed: &BTreeSet<String>,
    base: &WorkflowTopology,
    head: &WorkflowTopology,
) -> BTreeSet<String> {
    let mut owner = HashMap::new();
    let mut calls: HashMap<String, Vec<String>> = HashMap::new();
    for topology in [base, head] {
        for job in &topology.jobs {
            owner.insert(job.id.clone(), job.workflow_id.clone());
        }
        for edge in &topology.edges {
            if let WorkflowTopologyEdge::Calls(call) = edge {
                if let Some(target) = &call.to {
                    calls
                        .entry(target.clone())
                        .or_default()
                        .push(call.from.clone());
                }
            }
        }
    }
    let mut pending: Vec<String> = changed.iter().cloned().collect();
    let mut roots = BTreeSet::new();
    let mut seen = BTreeSet::new();
    while let Some(workflow) = pending.pop() {
        if !seen.insert(workflow.clone()) {
            continue;
        }
        for caller in calls.remove(&workflow).unwrap_or_default() {
            if owner.get(&caller).is_some_and(|value| value == entry) {
                roots.insert(caller);
            } else if let Some(parent) = owner.get(&caller) {
                pending.push(parent.clone());
            }
        }
    }
    roots
}

pub(super) fn action_root_callers(
    entry: &str,
    action_jobs: &BTreeSet<String>,
    base: &WorkflowTopology,
    head: &WorkflowTopology,
) -> BTreeSet<String> {
    let mut direct_roots = BTreeSet::new();
    let mut changed_workflows = BTreeSet::new();
    for topology in [base, head] {
        for job in &topology.jobs {
            if action_jobs.contains(&job.id) {
                if job.workflow_id == entry {
                    direct_roots.insert(job.id.clone());
                } else {
                    changed_workflows.insert(job.workflow_id.clone());
                }
            }
        }
    }
    direct_roots.extend(root_callers(entry, &changed_workflows, base, head));
    direct_roots
}

pub(super) fn reachable_workflows(
    entry: &str,
    base: &WorkflowTopology,
    head: &WorkflowTopology,
) -> BTreeSet<String> {
    let mut calls = HashMap::<String, Vec<String>>::new();
    let mut owners = HashMap::<String, String>::new();
    for topology in [base, head] {
        for job in &topology.jobs {
            owners.insert(job.id.clone(), job.workflow_id.clone());
        }
        for edge in &topology.edges {
            if let WorkflowTopologyEdge::Calls(call) = edge {
                if let (Some(target), Some(source)) = (&call.to, owners.get(&call.from)) {
                    calls
                        .entry(source.clone())
                        .or_default()
                        .push(target.clone());
                }
            }
        }
    }
    let mut seen = BTreeSet::new();
    let mut pending = vec![entry.to_string()];
    while let Some(path) = pending.pop() {
        if seen.insert(path.clone()) {
            pending.extend(calls.remove(&path).unwrap_or_default());
        }
    }
    seen
}
