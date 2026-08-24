use super::super::model::WorkflowTopology;
use super::diagnostics::topology_diagnostic;
use super::job_selection::{add_needs_closure, entry_job_ids};
use super::workflow_graph::{
    action_root_callers, affected_workflow_closure, reachable_workflows, root_callers,
};
use super::yaml::normalize_entry;
use super::{
    CiTopologyImpactDiagnostic, CiTopologyImpactDiagnosticScope, CiTopologyImpactReport,
    CI_TOPOLOGY_IMPACT_SCHEMA_VERSION,
};
use std::collections::BTreeSet;

pub(super) struct ImpactInputs<'a> {
    pub base_revision: String,
    pub head_revision: String,
    pub changed_paths: Vec<String>,
    pub entry_workflow: &'a str,
    pub base: &'a WorkflowTopology,
    pub head: &'a WorkflowTopology,
    pub reachable_actions: &'a BTreeSet<String>,
    pub changed_actions: &'a BTreeSet<String>,
    pub action_jobs: &'a BTreeSet<String>,
    pub changed_entry_jobs: &'a BTreeSet<String>,
    pub entry_global_change: bool,
    pub unowned_action: bool,
}

pub(super) fn project_impact(inputs: ImpactInputs<'_>) -> CiTopologyImpactReport {
    let ImpactInputs {
        base_revision,
        head_revision,
        changed_paths,
        entry_workflow,
        base,
        head,
        reachable_actions,
        changed_actions,
        action_jobs,
        changed_entry_jobs,
        entry_global_change,
        unowned_action,
    } = inputs;
    let mut diagnostics = Vec::new();
    let mut global_fallback = false;
    let entry = normalize_entry(entry_workflow);
    if !base
        .workflows
        .iter()
        .chain(&head.workflows)
        .any(|workflow| workflow.path == entry)
    {
        global_fallback = true;
        diagnostics.push(CiTopologyImpactDiagnostic {
            code: "missing-entry-workflow".into(),
            message: format!("entry workflow {entry} is absent from both revisions"),
            workflow_path: Some(entry.clone()),
            scope: CiTopologyImpactDiagnosticScope::Global,
            root_job_ids: None,
        });
    }
    let reachable = reachable_workflows(&entry, base, head);
    for diagnostic in base.diagnostics.iter().chain(&head.diagnostics) {
        // A duplicate workflow name may be reported on the newly conflicting
        // file rather than the entry-reachable reusable workflow it shadows.
        if reachable.contains(&diagnostic.workflow_path)
            || diagnostic.code.as_str() == "duplicate-workflow-name"
            || (diagnostic.code.as_str() == "malformed-workflow"
                && changed_paths.contains(&diagnostic.workflow_path))
        {
            let diagnostic = topology_diagnostic(diagnostic, &entry, base, head);
            global_fallback |= diagnostic.scope == CiTopologyImpactDiagnosticScope::Global;
            diagnostics.push(diagnostic);
        }
    }
    let known_workflow_paths = base
        .workflows
        .iter()
        .chain(&head.workflows)
        .map(|workflow| workflow.path.as_str())
        .collect::<BTreeSet<_>>();
    let changed_workflows = changed_paths
        .iter()
        .filter(|path| known_workflow_paths.contains(path.as_str()))
        .cloned()
        .collect::<BTreeSet<_>>();
    let reachable_changed_workflows = changed_workflows
        .iter()
        .filter(|path| reachable.contains(*path))
        .cloned()
        .collect::<BTreeSet<_>>();
    global_fallback |= entry_global_change;
    if unowned_action {
        global_fallback = true;
        diagnostics.push(CiTopologyImpactDiagnostic {
            code: "unowned-local-action".into(),
            message:
                "a changed local action does not resolve to an action descriptor in either revision"
                    .into(),
            workflow_path: None,
            scope: CiTopologyImpactDiagnosticScope::Global,
            root_job_ids: None,
        });
    }
    let action_roots = action_root_callers(&entry, action_jobs, base, head);
    if !reachable_actions.is_disjoint(changed_actions) && action_roots.is_empty() {
        global_fallback = true;
        diagnostics.push(CiTopologyImpactDiagnostic {
            code: "unresolved-local-action-caller".into(),
            message: "a changed reachable local action has no resolvable entry-workflow caller"
                .into(),
            workflow_path: None,
            scope: CiTopologyImpactDiagnosticScope::Global,
            root_job_ids: None,
        });
    }
    let diagnostic_roots = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.scope == CiTopologyImpactDiagnosticScope::Localized)
        .flat_map(|diagnostic| diagnostic.root_job_ids.iter().flatten().cloned())
        .collect::<BTreeSet<_>>();
    let mut jobs = if global_fallback {
        // A deleted or malformed head `jobs` mapping has no head roots to
        // enumerate. Keep the base union so fail-open never becomes empty.
        base.jobs
            .iter()
            .chain(&head.jobs)
            .filter(|job| job.workflow_id == entry)
            .map(|job| job.id.clone())
            .collect()
    } else {
        let mut roots = root_callers(&entry, &reachable_changed_workflows, base, head);
        roots.extend(entry_job_ids(&entry, changed_entry_jobs, base, head));
        if !reachable_actions.is_disjoint(changed_actions) {
            roots.extend(action_roots);
        }
        roots.extend(diagnostic_roots);
        roots
    };
    add_needs_closure(&mut jobs, base, head);
    let mut affected_workflows = affected_workflow_closure(&changed_workflows, base, head);
    let action_workflows = base
        .jobs
        .iter()
        .chain(&head.jobs)
        .filter(|job| action_jobs.contains(&job.id))
        .map(|job| job.workflow_id.clone())
        .filter(|workflow| reachable.contains(workflow))
        .collect();
    affected_workflows.extend(affected_workflow_closure(&action_workflows, base, head));
    diagnostics.sort_by(|left, right| {
        (
            left.code.as_str(),
            left.workflow_path.as_deref(),
            left.message.as_str(),
        )
            .cmp(&(
                right.code.as_str(),
                right.workflow_path.as_deref(),
                right.message.as_str(),
            ))
    });
    diagnostics.dedup_by(|left, right| {
        left.code == right.code
            && left.workflow_path == right.workflow_path
            && left.message == right.message
    });
    CiTopologyImpactReport {
        schema_version: CI_TOPOLOGY_IMPACT_SCHEMA_VERSION,
        base_revision,
        head_revision,
        changed_paths,
        affected_workflows: affected_workflows.into_iter().collect(),
        affected_root_job_ids: jobs.into_iter().collect(),
        diagnostics,
        global_fallback,
    }
}
