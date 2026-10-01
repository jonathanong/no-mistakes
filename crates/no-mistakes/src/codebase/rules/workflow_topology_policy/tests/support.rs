use crate::codebase::ci_graph::permissions::{PermissionSource, ResolvedPermissions};
use crate::codebase::workflow_topology::model::{
    ConcurrencyEffective, ConcurrencyRaw, ConcurrencyValue, JobKind, WorkflowConcurrency,
    WorkflowJobNode, WorkflowNode, WorkflowTopology,
};
use std::collections::BTreeMap;

pub(super) fn concurrency(
    group: &str,
    cancel: ConcurrencyValue,
    queue: &str,
) -> WorkflowConcurrency {
    WorkflowConcurrency {
        raw: ConcurrencyRaw {
            group: group.to_string(),
            cancel_in_progress: Some(cancel.clone()),
            queue: Some(queue.to_string()),
        },
        effective: ConcurrencyEffective {
            group: group.to_string(),
            cancel_in_progress: cancel,
            queue: queue.to_string(),
        },
    }
}

pub(super) fn workflow(
    path: &str,
    concurrency: Option<WorkflowConcurrency>,
    job_keys: &[&str],
) -> WorkflowNode {
    WorkflowNode {
        id: path.to_string(),
        path: path.to_string(),
        name: path.to_string(),
        callable: false,
        workflow_call: None,
        triggers: Vec::new(),
        job_ids: job_keys.iter().map(|key| format!("{path}#{key}")).collect(),
        concurrency,
        env: None,
        secret_references: None,
    }
}

pub(super) fn job(
    path: &str,
    key: &str,
    concurrency: Option<WorkflowConcurrency>,
) -> WorkflowJobNode {
    WorkflowJobNode {
        id: format!("{path}#{key}"),
        workflow_id: path.to_string(),
        key: key.to_string(),
        kind: JobKind::Job,
        name: None,
        condition: None,
        matrix: None,
        concurrency,
        steps: Vec::new(),
        environment: None,
        timeout_minutes: None,
        runs_on: None,
        permissions: ResolvedPermissions {
            source: PermissionSource::Default,
            scopes: BTreeMap::new(),
            assumed_default: true,
        },
        outputs: None,
        env: None,
        secret_references: None,
    }
}

pub(super) fn topology(
    workflows: Vec<WorkflowNode>,
    jobs: Vec<WorkflowJobNode>,
) -> WorkflowTopology {
    WorkflowTopology {
        schema_version: 1,
        workflows,
        jobs,
        edges: Vec::new(),
        diagnostics: Vec::new(),
    }
}
