use super::super::call_contract_diagnostics::diagnose_workflow_call_contracts;
use super::super::model::{
    DiagnosticCode, JobKind, JsonScalar, WorkflowCallBindings, WorkflowCallContract,
    WorkflowCallEdge, WorkflowCallInput, WorkflowCallInputType, WorkflowCallOutput,
    WorkflowCallSecret, WorkflowCallSecretsBinding, WorkflowJobNode, WorkflowNode,
    WorkflowTopologyEdge,
};
use super::super::parse::ParsedWorkflowOutputReference;
use crate::codebase::ci_graph::permissions::{PermissionSource, ResolvedPermissions};
use std::collections::BTreeMap;

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

fn input(input_type: Option<WorkflowCallInputType>, required: bool) -> WorkflowCallInput {
    WorkflowCallInput {
        input_type,
        required,
        default: None,
        description: None,
    }
}

fn inherit_call(from: &str, target: &str, local: bool, to: Option<String>) -> WorkflowTopologyEdge {
    WorkflowTopologyEdge::Calls(WorkflowCallEdge {
        from: from.into(),
        target: target.into(),
        local,
        bindings: WorkflowCallBindings {
            inputs: BTreeMap::new(),
            secrets: WorkflowCallSecretsBinding::Inherit,
        },
        to,
    })
}

fn output_ref(job: &str, call_job_key: &str, output: &str) -> ParsedWorkflowOutputReference {
    ParsedWorkflowOutputReference {
        consumer_job_id: job.into(),
        call_job_key: call_job_key.into(),
        output: output.into(),
    }
}

#[test]
fn call_contract_diagnostics_cover_type_inherit_optional_and_output_duplicates() {
    let callee_path = ".github/workflows/callee.yml";
    let mut contract = WorkflowCallContract::default();
    contract.inputs.insert(
        "name".into(),
        input(Some(WorkflowCallInputType::String), true),
    );
    contract.inputs.insert(
        "flag".into(),
        input(Some(WorkflowCallInputType::Boolean), false),
    );
    contract.inputs.insert(
        "title".into(),
        input(Some(WorkflowCallInputType::String), true),
    );
    contract.inputs.insert(
        "count".into(),
        input(Some(WorkflowCallInputType::Number), true),
    );
    contract.inputs.insert(
        "opaque".into(),
        input(Some(WorkflowCallInputType::String), true),
    );
    contract.inputs.insert("any".into(), input(None, true));
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
    let remote = workflow_node(".github/workflows/remote.yml", true);
    let uncallable = workflow_node(".github/workflows/plain.yml", false);
    let remote_path = remote.path.clone();
    let uncallable_path = uncallable.path.clone();
    let call = WorkflowTopologyEdge::Calls(WorkflowCallEdge {
        from: ".github/workflows/ci.yml#build".into(),
        target: "./callee.yml".into(),
        local: true,
        bindings: WorkflowCallBindings {
            inputs: BTreeMap::from([
                (
                    "name".into(),
                    JsonScalar::Number(serde_json::Number::from(1)),
                ),
                ("title".into(), JsonScalar::Text("a".into())),
                ("Title".into(), JsonScalar::Text("b".into())),
                ("count".into(), JsonScalar::Bool(true)),
                ("opaque".into(), JsonScalar::Text("${{ inputs.x }}".into())),
                ("any".into(), JsonScalar::Bool(true)),
            ]),
            secrets: WorkflowCallSecretsBinding::Inherit,
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
        &[caller, callee, remote, uncallable],
        std::slice::from_ref(&job),
        &[
            call,
            inherit_call(
                ".github/workflows/ci.yml#remote",
                "org/repo/.github/workflows/reuse.yml@v1",
                false,
                Some(remote_path),
            ),
            inherit_call(
                ".github/workflows/ci.yml#plain",
                "./plain.yml",
                true,
                Some(uncallable_path),
            ),
        ],
        &[
            output_ref(".github/workflows/ci.yml#build", "build", "missing"),
            output_ref(".github/workflows/ci.yml#build", "build", "missing"),
            output_ref(".github/workflows/ci.yml#build", "build", "ready"),
            output_ref(".github/workflows/ci.yml#build", "other", "missing"),
        ],
        &mut diagnostics,
    );
    let codes: Vec<_> = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect();
    assert!(codes.contains(&DiagnosticCode::WorkflowCallInputTypeMismatch));
    assert!(codes.contains(&DiagnosticCode::UnknownWorkflowCallOutput));
    assert!(!codes.contains(&DiagnosticCode::MissingWorkflowCallSecret));
}
