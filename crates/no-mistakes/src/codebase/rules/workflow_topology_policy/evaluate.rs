use super::{finding, Options};
use crate::codebase::rules::RuleFinding;
use crate::codebase::workflow_topology::model::WorkflowTopology;
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct Index<'a> {
    pub(super) jobs:
        BTreeMap<&'a str, &'a crate::codebase::workflow_topology::model::WorkflowJobNode>,
    pub(super) workflows:
        BTreeMap<&'a str, &'a crate::codebase::workflow_topology::model::WorkflowNode>,
    downstream: BTreeMap<&'a str, BTreeSet<&'a str>>,
    upstream: BTreeMap<&'a str, BTreeSet<&'a str>>,
    caller_jobs: BTreeMap<&'a str, BTreeSet<&'a str>>,
}

impl<'a> Index<'a> {
    pub(super) fn new(topology: &'a WorkflowTopology) -> Self {
        let jobs: BTreeMap<&str, _> = topology
            .jobs
            .iter()
            .map(|job| (job.id.as_str(), job))
            .collect();
        let workflows: BTreeMap<&str, _> = topology
            .workflows
            .iter()
            .map(|workflow| (workflow.path.as_str(), workflow))
            .collect();
        let mut downstream: BTreeMap<&str, BTreeSet<&str>> = jobs
            .keys()
            .copied()
            .map(|id| (id, BTreeSet::new()))
            .collect();
        let mut upstream = downstream.clone();
        let mut caller_jobs: BTreeMap<&str, BTreeSet<&str>> = workflows
            .keys()
            .copied()
            .map(|path| (path, BTreeSet::new()))
            .collect();
        for edge in &topology.edges {
            match edge {
                crate::codebase::workflow_topology::model::WorkflowTopologyEdge::Needs(needs) => {
                    if let Some(set) = downstream.get_mut(needs.from.as_str()) {
                        set.insert(needs.to.as_str());
                    }
                    if let Some(set) = upstream.get_mut(needs.to.as_str()) {
                        set.insert(needs.from.as_str());
                    }
                }
                crate::codebase::workflow_topology::model::WorkflowTopologyEdge::Calls(call)
                    if call.local =>
                {
                    if let Some(to) = call.to.as_deref() {
                        if let Some(set) = caller_jobs.get_mut(to) {
                            set.insert(call.from.as_str());
                        }
                    }
                }
                _ => {}
            }
        }
        Self {
            jobs,
            workflows,
            downstream,
            upstream,
            caller_jobs,
        }
    }

    pub(super) fn direct_downstream(&self, job: &str) -> Vec<&str> {
        sorted(self.downstream.get(job))
    }

    pub(super) fn transitive_downstream(&self, job: &str) -> Vec<&str> {
        let mut visited = BTreeSet::new();
        let mut pending: Vec<&str> = self.direct_downstream(job);
        while let Some(current) = pending.pop() {
            if current == job || !visited.insert(current) {
                continue;
            }
            pending.extend(self.direct_downstream(current));
        }
        visited.into_iter().collect()
    }

    pub(super) fn direct_upstream(&self, job: &str) -> Vec<&str> {
        sorted(self.upstream.get(job))
    }

    pub(super) fn direct_caller_jobs(&self, workflow: &str) -> Vec<&str> {
        sorted(self.caller_jobs.get(workflow))
    }
}

fn sorted<'a>(set: Option<&BTreeSet<&'a str>>) -> Vec<&'a str> {
    set.map(|values| values.iter().copied().collect())
        .unwrap_or_default()
}

pub(super) fn lint(topology: &WorkflowTopology, opts: &Options) -> Vec<RuleFinding> {
    let index = Index::new(topology);
    let mut findings = Vec::new();
    if !opts.job_inventory.is_empty() {
        findings.extend(inventory(topology, opts));
    }
    findings.extend(job_presence(&index, opts));
    findings.extend(super::evaluate_graph::lint(topology, &index, opts));
    findings.extend(super::evaluate_steps::lint(topology, &index, opts));
    findings
}

fn inventory(topology: &WorkflowTopology, opts: &Options) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    let mut actual = BTreeMap::new();
    for workflow in &topology.workflows {
        let keys: Vec<String> = workflow
            .job_ids
            .iter()
            .filter_map(|id| id.rsplit_once('#').map(|(_, key)| key.to_string()))
            .collect();
        actual.insert(workflow.path.clone(), keys);
    }
    for (path, keys) in &actual {
        match opts.job_inventory.get(path) {
            None => findings.push(finding(format!("workflow inventory missing: {path}"))),
            Some(expected) => {
                let mut expected = expected.clone();
                expected.sort();
                let mut got = keys.clone();
                got.sort();
                if expected != got {
                    findings.push(finding(format!(
                        "job inventory mismatch: {path}: expected {}, got {}",
                        expected.join(", "),
                        got.join(", ")
                    )));
                }
            }
        }
    }
    for path in opts.job_inventory.keys() {
        if !actual.contains_key(path) {
            findings.push(finding(format!("workflow inventory stale: {path}")));
        }
    }
    findings
}

fn job_presence(index: &Index<'_>, opts: &Options) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for id in &opts.required_jobs {
        if !index.jobs.contains_key(id.as_str()) {
            findings.push(finding(format!("required job missing: {id}")));
        }
    }
    for id in &opts.forbidden_jobs {
        if index.jobs.contains_key(id.as_str()) {
            findings.push(finding(format!("forbidden job present: {id}")));
        }
    }
    findings
}
