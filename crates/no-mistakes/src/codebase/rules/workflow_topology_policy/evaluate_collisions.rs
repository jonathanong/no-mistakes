use super::finding;
use crate::codebase::rules::RuleFinding;
use crate::codebase::workflow_topology::model::WorkflowTopology;
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

pub(super) fn lint(topology: &WorkflowTopology) -> Vec<RuleFinding> {
    let mut groups: BTreeMap<(String, String), BTreeSet<String>> = BTreeMap::new();
    for owner in owners(topology) {
        let partition = if workflow_scoped(&owner.group) {
            owner.workflow_path
        } else {
            String::new()
        };
        groups
            .entry((partition, owner.group.to_lowercase()))
            .or_default()
            .insert(owner.id);
    }
    let mut findings = Vec::new();
    for ((_, lowered), ids) in groups {
        if ids.len() < 2 {
            continue;
        }
        findings.push(finding(format!(
            "concurrency group collision: {lowered}: {}",
            ids.into_iter().collect::<Vec<_>>().join(", ")
        )));
    }
    findings
}

struct Owner {
    id: String,
    workflow_path: String,
    group: String,
}

fn owners(topology: &WorkflowTopology) -> Vec<Owner> {
    let mut owners = Vec::new();
    for workflow in &topology.workflows {
        if let Some(concurrency) = &workflow.concurrency {
            owners.push(Owner {
                id: workflow.path.clone(),
                workflow_path: workflow.path.clone(),
                group: concurrency.effective.group.clone(),
            });
        }
    }
    for job in &topology.jobs {
        if let Some(concurrency) = &job.concurrency {
            owners.push(Owner {
                id: job.id.clone(),
                workflow_path: job.workflow_id.clone(),
                group: concurrency.effective.group.clone(),
            });
        }
    }
    owners
}

/// Only the exact `${{ github.workflow }}` placeholder partitions by file.
/// A longer expression such as `${{ github.workflow || 'x' }}` does not.
fn workflow_scoped(group: &str) -> bool {
    workflow_placeholder().is_match(group)
}

fn workflow_placeholder() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r"\$\{\{\s*github\.workflow\s*\}\}").expect("github.workflow placeholder")
    })
}
