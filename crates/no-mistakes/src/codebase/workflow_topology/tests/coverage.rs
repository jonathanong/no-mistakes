use super::super::expression_references::static_references;
use super::super::load_workflow_topology;
use super::super::model::{
    ConcurrencyEffective, ConcurrencyRaw, ConcurrencyValue, DiagnosticCode, JobKind, JsonScalar,
    WorkflowCallBindings, WorkflowCallEdge, WorkflowCallSecretsBinding, WorkflowConcurrency,
    WorkflowJobNode, WorkflowNode, WorkflowRunEdge, WorkflowTopology, WorkflowTopologyEdge,
};
use super::super::render_mermaid::render_workflow_topology_mermaid;
use crate::codebase::ci_graph::permissions::{PermissionSource, ResolvedPermissions};
use crate::config::v2::schema::CiConfig;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/workflow-topology")
            .join(name),
    )
}

fn topology(name: &str) -> WorkflowTopology {
    load_workflow_topology(&fixture(name), &CiConfig::default(), &[])
}

#[test]
fn mermaid_renders_needs_artifact_and_matrix_job_shapes() {
    let needs = render_workflow_topology_mermaid(&topology("needs-basic"));
    assert!(needs.contains("-->"), "{needs}");

    let artifacts = render_workflow_topology_mermaid(&topology("artifact-basic"));
    assert!(artifacts.contains("artifact:"), "{artifacts}");

    let matrix = render_workflow_topology_mermaid(&topology("artifact-matrix"));
    assert!(matrix.contains("[matrix]"), "{matrix}");
}

#[test]
fn mermaid_renders_expression_cancel_in_progress() {
    let mermaid = render_workflow_topology_mermaid(&WorkflowTopology {
        schema_version: 1,
        workflows: vec![WorkflowNode {
            id: "ci.yml".into(),
            path: "ci.yml".into(),
            name: "CI".into(),
            callable: false,
            workflow_call: None,
            triggers: Vec::new(),
            job_ids: vec!["ci.yml#build".into()],
            concurrency: Some(WorkflowConcurrency {
                raw: ConcurrencyRaw {
                    group: "ci".into(),
                    cancel_in_progress: Some(ConcurrencyValue::Text("${{ inputs.cancel }}".into())),
                    queue: None,
                },
                effective: ConcurrencyEffective {
                    group: "ci".into(),
                    cancel_in_progress: ConcurrencyValue::Text("${{ inputs.cancel }}".into()),
                    queue: "single".into(),
                },
            }),
            env: None,
            secret_references: None,
        }],
        jobs: vec![WorkflowJobNode {
            id: "ci.yml#build".into(),
            workflow_id: "ci.yml".into(),
            key: "build".into(),
            kind: JobKind::Job,
            name: None,
            condition: None,
            matrix: None,
            concurrency: Some(WorkflowConcurrency {
                raw: ConcurrencyRaw {
                    group: "job".into(),
                    cancel_in_progress: Some(ConcurrencyValue::Bool(true)),
                    queue: None,
                },
                effective: ConcurrencyEffective {
                    group: "job".into(),
                    cancel_in_progress: ConcurrencyValue::Bool(true),
                    queue: "single".into(),
                },
            }),
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
        }],
        edges: Vec::new(),
        diagnostics: Vec::new(),
    });
    assert!(mermaid.contains("cancel=${{ inputs.cancel }}"), "{mermaid}");
}

#[test]
fn static_references_tolerate_whitespace_after_needs() {
    let refs = static_references(Some("needs [ 'build-job' ]"), "needs");
    assert_eq!(refs, vec!["build-job".to_string()]);
}

#[test]
fn workflow_run_self_cycle_is_diagnosed() {
    let report = topology("workflow-run-self-cycle");
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == DiagnosticCode::WorkflowRunCycle));
}

#[test]
fn mermaid_renders_remote_calls_workflow_run_and_escaped_labels() {
    let mermaid = render_workflow_topology_mermaid(&WorkflowTopology {
        schema_version: 1,
        workflows: vec![WorkflowNode {
            id: "ci.yml".into(),
            path: "ci.yml".into(),
            name: "CI & \"build\" <prod>".into(),
            callable: false,
            workflow_call: None,
            triggers: Vec::new(),
            job_ids: vec!["ci.yml#build".into()],
            concurrency: None,
            env: None,
            secret_references: None,
        }],
        jobs: vec![WorkflowJobNode {
            id: "ci.yml#build".into(),
            workflow_id: "ci.yml".into(),
            key: "build".into(),
            kind: JobKind::Job,
            name: Some("Build #1".into()),
            condition: None,
            matrix: None,
            concurrency: None,
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
        }],
        edges: vec![
            WorkflowTopologyEdge::Calls(WorkflowCallEdge {
                from: "ci.yml#build".into(),
                target: "org/repo/.github/workflows/reuse.yml@v1".into(),
                local: false,
                bindings: WorkflowCallBindings {
                    inputs: BTreeMap::from([("ref".into(), JsonScalar::Text("main".into()))]),
                    secrets: WorkflowCallSecretsBinding::Inherit,
                },
                to: None,
            }),
            WorkflowTopologyEdge::WorkflowRun(WorkflowRunEdge {
                from: "ci.yml".into(),
                to: "ci.yml".into(),
                types: None,
                branches: None,
                branches_ignore: None,
            }),
        ],
        diagnostics: Vec::new(),
    });
    assert!(mermaid.contains("calls"), "{mermaid}");
    assert!(mermaid.contains("workflow_run"), "{mermaid}");
    assert!(mermaid.contains("&amp;"), "{mermaid}");
    assert!(mermaid.contains("&#35;"), "{mermaid}");
}
