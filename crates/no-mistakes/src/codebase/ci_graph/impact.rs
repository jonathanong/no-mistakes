//! `no-mistakes ci impact <file>` — map changed files to the workflows they
//! trigger, with each job's resolved permissions.

use super::model::CiWarning;
use super::permissions::{effective_permissions, ResolvedPermissions};
use super::triggers::{evaluate_trigger, MatchedFilter, TriggerMatch};
use super::WorkflowSet;
use serde::Serialize;

/// Result of an impact query for one or more changed files.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CiImpactReport {
    /// The changed files evaluated (repo-relative, slash-normalized).
    pub changed_files: Vec<String>,
    /// Impacted workflows, sorted by path.
    pub workflows: Vec<ImpactedWorkflow>,
    /// Non-fatal load/parse warnings.
    pub warnings: Vec<CiWarning>,
}

/// A workflow triggered by the changed files.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ImpactedWorkflow {
    /// Repo-relative, slash-normalized path.
    pub path: String,
    /// Top-level `name:`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Why the workflow is impacted.
    pub trigger: TriggerMatch,
    /// True for reusable workflows (`on: workflow_call`).
    pub reusable: bool,
    /// Filters that matched (empty for `always`).
    pub matched_filters: Vec<MatchedFilter>,
    /// Jobs with their resolved permissions.
    pub jobs: Vec<ImpactedJob>,
}

/// A job within an impacted workflow.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ImpactedJob {
    /// Job id.
    pub id: String,
    /// `name:` if present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Reusable-workflow call target, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uses: Option<String>,
    /// Resolved permissions for the job.
    pub permissions: ResolvedPermissions,
}

/// Compute which workflows the `changed_files` trigger.
pub fn analyze_impact(set: &WorkflowSet, changed_files: &[String]) -> CiImpactReport {
    let mut workflows = Vec::new();

    for workflow in &set.workflows {
        let mut any_matched = false;
        let mut any_always = false;
        let mut matched_filters: Vec<MatchedFilter> = Vec::new();

        for file in changed_files {
            match evaluate_trigger(workflow, file) {
                (TriggerMatch::Matched, filters) => {
                    any_matched = true;
                    for filter in filters {
                        if !matched_filters.contains(&filter) {
                            matched_filters.push(filter);
                        }
                    }
                }
                (TriggerMatch::Always, _) => any_always = true,
                (TriggerMatch::NotMatched, _) | (TriggerMatch::NoPathEvents, _) => {}
            }
        }

        let trigger = if any_matched {
            TriggerMatch::Matched
        } else if any_always {
            TriggerMatch::Always
        } else {
            continue;
        };

        matched_filters.sort_by(|a, b| (&a.event, &a.pattern).cmp(&(&b.event, &b.pattern)));

        let jobs = workflow
            .jobs
            .iter()
            .map(|job| ImpactedJob {
                id: job.id.clone(),
                name: job.name.clone(),
                uses: job.uses.clone(),
                permissions: effective_permissions(workflow, job),
            })
            .collect();

        workflows.push(ImpactedWorkflow {
            path: workflow.path.clone(),
            name: workflow.name.clone(),
            trigger,
            reusable: workflow.is_reusable,
            matched_filters,
            jobs,
        });
    }

    workflows.sort_by(|a, b| a.path.cmp(&b.path));
    CiImpactReport {
        changed_files: changed_files.to_vec(),
        workflows,
        warnings: set.warnings.clone(),
    }
}
