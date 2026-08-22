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

fn workflow_node(path: &str, callable: bool) -> WorkflowNode {
    WorkflowNode {
        id: path.into(),
        path: path.into(),
        name: path.into(),
        callable,
        workflow_call: None,
        triggers: Vec::new(),
        job_ids: Vec::new(),
        concurrency: None,
        env: None,
        secret_references: None,
    }
}

#[test]
fn workflow_filters_cover_basename_invalid_and_callee_selection() {
    use super::super::topology_graph::{
        diagnose_workflow_filters, edge_belongs_to_selection, select_workflow_paths,
        WORKFLOWS_DIRECTORY,
    };
    use std::collections::{HashMap, HashSet};

    let ci = ".github/workflows/ci.yml";
    let callee = ".github/workflows/callee.yml";
    let mut workflows = HashMap::new();
    workflows.insert(ci.to_string(), workflow_node(ci, false));
    workflows.insert(callee.to_string(), workflow_node(callee, true));
    let dirs = [WORKFLOWS_DIRECTORY.to_string()];
    let all = select_workflow_paths(&[], &workflows, &[], &dirs);
    assert_eq!(all.len(), 2);

    let selected = select_workflow_paths(&[String::from("ci.yml")], &workflows, &[], &dirs);
    assert!(selected.contains(ci));

    let call = WorkflowTopologyEdge::Calls(WorkflowCallEdge {
        from: format!("{ci}#job"),
        target: "./callee.yml".into(),
        local: true,
        bindings: WorkflowCallBindings {
            inputs: BTreeMap::new(),
            secrets: WorkflowCallSecretsBinding::Inherit,
        },
        to: Some(callee.into()),
    });
    let with_callee = select_workflow_paths(
        &[String::from("ci.yml")],
        &workflows,
        std::slice::from_ref(&call),
        &dirs,
    );
    assert!(with_callee.contains(callee));

    let mut diagnostics = Vec::new();
    diagnose_workflow_filters(
        &[
            String::from("/abs.yml"),
            String::from(".."),
            String::from("missing.yml"),
            String::from("./.github/workflows/ci.yml"),
        ],
        &workflows,
        &mut diagnostics,
        &dirs,
    );
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == DiagnosticCode::InvalidWorkflowFilter));
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == DiagnosticCode::UnknownWorkflowFilter));

    let mut selected = HashSet::new();
    selected.insert(ci.to_string());
    assert!(edge_belongs_to_selection(&call, &selected));
    assert!(!edge_belongs_to_selection(
        &WorkflowTopologyEdge::Needs(super::super::model::NeedsEdge {
            from: ci.into(),
            to: callee.into(),
        }),
        &selected
    ));
}

#[test]
fn call_contract_diagnostics_cover_input_secret_and_output_mismatches() {
    use super::super::call_contract_diagnostics::diagnose_workflow_call_contracts;
    use super::super::model::{
        JsonScalar, WorkflowCallContract, WorkflowCallInput, WorkflowCallInputType,
        WorkflowCallOutput, WorkflowCallSecret, WorkflowCallSecretsBinding,
    };
    use super::super::parse::ParsedWorkflowOutputReference;

    let callee_path = ".github/workflows/callee.yml";
    let mut contract = WorkflowCallContract::default();
    contract.inputs.insert(
        "name".into(),
        WorkflowCallInput {
            input_type: Some(WorkflowCallInputType::String),
            required: true,
            default: None,
            description: None,
        },
    );
    contract.secrets.insert(
        "token".into(),
        WorkflowCallSecret {
            required: true,
            description: None,
        },
    );
    contract.outputs.insert(
        "ready".into(),
        WorkflowCallOutput {
            value: Some("true".into()),
            description: None,
        },
    );
    let mut callee = workflow_node(callee_path, true);
    callee.workflow_call = Some(contract);
    let caller = workflow_node(".github/workflows/ci.yml", false);
    let call = WorkflowTopologyEdge::Calls(WorkflowCallEdge {
        from: ".github/workflows/ci.yml#build".into(),
        target: "./callee.yml".into(),
        local: true,
        bindings: WorkflowCallBindings {
            inputs: BTreeMap::from([(
                "extra".into(),
                JsonScalar::Number(serde_json::Number::from(1)),
            )]),
            secrets: WorkflowCallSecretsBinding::Explicit {
                values: BTreeMap::from([("unknown".into(), JsonScalar::Text("x".into()))]),
            },
        },
        to: Some(callee_path.into()),
    });
    let job = WorkflowJobNode {
        id: ".github/workflows/ci.yml#build".into(),
        workflow_id: ".github/workflows/ci.yml".into(),
        key: "build".into(),
        kind: JobKind::Job,
        name: None,
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
    };
    let mut diagnostics = Vec::new();
    diagnose_workflow_call_contracts(
        &[caller, callee],
        std::slice::from_ref(&job),
        std::slice::from_ref(&call),
        &[ParsedWorkflowOutputReference {
            consumer_job_id: ".github/workflows/ci.yml#build".into(),
            call_job_key: "build".into(),
            output: "missing".into(),
        }],
        &mut diagnostics,
    );
    let codes: Vec<_> = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect();
    assert!(codes.contains(&DiagnosticCode::MissingWorkflowCallInput));
    assert!(codes.contains(&DiagnosticCode::UnknownWorkflowCallInput));
    assert!(codes.contains(&DiagnosticCode::MissingWorkflowCallSecret));
    assert!(codes.contains(&DiagnosticCode::UnknownWorkflowCallSecret));
    assert!(codes.contains(&DiagnosticCode::UnknownWorkflowCallOutput));
}

#[test]
fn value_primitives_cover_list_scalar_and_lookup_fallbacks() {
    use super::super::value_primitives::{
        concurrency_value, is_record, string_list, string_value, OrderedJson,
    };
    assert!(string_list(None).is_empty());
    assert_eq!(
        string_list(Some(&serde_yaml::Value::String("one".into()))),
        vec!["one".to_string()]
    );
    assert_eq!(
        string_list(Some(&serde_yaml::Value::Sequence(vec![
            serde_yaml::Value::String("keep".into()),
            serde_yaml::Value::Bool(true),
        ]))),
        vec!["keep".to_string()]
    );
    assert_eq!(
        string_value(Some(&serde_yaml::Value::Bool(true))).as_deref(),
        Some("true")
    );
    assert_eq!(
        string_value(Some(&serde_yaml::Value::Number(2.into()))).as_deref(),
        Some("2")
    );
    assert!(string_value(Some(&serde_yaml::Value::Mapping(Default::default()))).is_none());
    assert!(is_record(Some(&serde_yaml::Value::Mapping(
        Default::default()
    ))));
    assert!(matches!(
        concurrency_value(Some(&serde_yaml::Value::Bool(false))),
        Some(ConcurrencyValue::Bool(false))
    ));
    let json = OrderedJson::Bool(true);
    assert!(json.get("missing").is_none());
    assert!(json.as_str().is_none());
}
