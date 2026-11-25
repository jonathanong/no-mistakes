use super::call_bindings_map::{
    check_declared_map, job_field_finding, unanalyzable_job_field, BindingCheck,
};
use super::call_bindings_yaml::RawWorkflows;
use super::evaluate::Index;
use super::{workflow_finding, LocatedFinding, Options};
use crate::codebase::ci_workflows::PreparedWorkflowTopologyInputs;
use crate::codebase::workflow_topology::model::{
    DiagnosticCode, WorkflowCallContract, WorkflowCallEdge, WorkflowCallSecretsBinding,
    WorkflowTopologyEdge,
};

#[derive(Default)]
pub(super) struct CallBindingOutput {
    pub(super) findings: Vec<LocatedFinding>,
    pub(super) unanalyzable: Vec<LocatedFinding>,
}

pub(super) fn lint(inputs: &PreparedWorkflowTopologyInputs, options: &Options) -> CallBindingOutput {
    let mut out = CallBindingOutput::default();
    if !options.require_explicit_workflow_call_bindings {
        return out;
    }
    let topology = inputs.topology();
    let index = Index::new(topology);
    let raw = RawWorkflows::from_parsed(&inputs.workflow_documents);
    for diagnostic in &topology.diagnostics {
        if diagnostic.code == DiagnosticCode::MalformedWorkflow {
            let mut finding = workflow_finding(
                &diagnostic.workflow_path,
                format!(
                    "cannot verify reusable workflow call bindings because `{}` could not be parsed ({}); repair the workflow YAML or suppress this file",
                    diagnostic.workflow_path, diagnostic.message
                ),
            );
            finding.finding.target = Some("unanalyzable".to_owned());
            out.unanalyzable.push(finding);
        }
    }
    for edge in &topology.edges {
        let WorkflowTopologyEdge::Calls(call) = edge else {
            continue;
        };
        check_call(call, &index, &raw, options, &mut out);
    }
    out
}

fn check_call(
    call: &WorkflowCallEdge,
    index: &Index<'_>,
    raw: &RawWorkflows,
    options: &Options,
    out: &mut CallBindingOutput,
) {
    let Some((caller_path, job_key)) = call.from.rsplit_once('#') else {
        return;
    };
    let is_inherit = matches!(call.bindings.secrets, WorkflowCallSecretsBinding::Inherit);
    if is_inherit && !options.forbid_secrets_inherit {
        out.findings.push(job_field_finding(
            &call.from,
            job_key,
            "secrets",
            "forbidden secrets: inherit; map each secret explicitly on the reusable workflow call"
                .to_string(),
        ));
    }
    if !call.local {
        if !is_inherit {
            out.unanalyzable.push(unanalyzable_job_field(
                &call.from,
                job_key,
                "uses",
                format!(
                    "reusable workflow call `{}` targets remote workflow `{}`; declared secrets and required inputs cannot be verified locally",
                    call.from, call.target
                ),
            ));
        }
        return;
    }
    let Some(callee_path) = call.to.as_deref() else {
        return;
    };
    let Some(callee) = index.workflows.get(callee_path) else {
        out.unanalyzable.push(unanalyzable_job_field(
            &call.from,
            job_key,
            "uses",
            format!(
                "reusable workflow call `{}` targets `{callee_path}`, which is missing or could not be parsed; declared secrets and required inputs cannot be verified",
                call.from
            ),
        ));
        return;
    };
    let Some(contract) = &callee.workflow_call else {
        out.unanalyzable.push(unanalyzable_job_field(
            &call.from,
            job_key,
            "uses",
            format!(
                "reusable workflow call `{}` targets `{callee_path}`, which does not declare `on.workflow_call`; declared secrets and required inputs cannot be verified",
                call.from
            ),
        ));
        return;
    };
    if !is_inherit {
        check_secrets(call, caller_path, job_key, callee_path, contract, raw, out);
    }
    check_inputs(call, caller_path, job_key, callee_path, contract, raw, out);
}

fn check_secrets(
    call: &WorkflowCallEdge,
    caller_path: &str,
    job_key: &str,
    callee_path: &str,
    contract: &WorkflowCallContract,
    raw: &RawWorkflows,
    out: &mut CallBindingOutput,
) {
    if let Some(shape) = raw.callee_contract_field_non_mapping(callee_path, "secrets") {
        out.unanalyzable.push(unanalyzable_job_field(
            &call.from,
            job_key,
            "uses",
            format!(
                "reusable workflow `{callee_path}` sets `on.workflow_call.secrets` to `{shape}`, which cannot be verified statically"
            ),
        ));
        return;
    }
    let declared: Vec<&str> = contract.secrets.keys().map(String::as_str).collect();
    check_declared_map(
        BindingCheck {
            call_from: &call.from,
            caller_path,
            job_key,
            callee_path,
            field: "secrets",
            noun: "declared secret",
            action: "map",
            declared: &declared,
        },
        raw,
        &mut out.findings,
        &mut out.unanalyzable,
    );
}

fn check_inputs(
    call: &WorkflowCallEdge,
    caller_path: &str,
    job_key: &str,
    callee_path: &str,
    contract: &WorkflowCallContract,
    raw: &RawWorkflows,
    out: &mut CallBindingOutput,
) {
    if let Some(shape) = raw.callee_contract_field_non_mapping(callee_path, "inputs") {
        out.unanalyzable.push(unanalyzable_job_field(
            &call.from,
            job_key,
            "uses",
            format!(
                "reusable workflow `{callee_path}` sets `on.workflow_call.inputs` to `{shape}`, which cannot be verified statically"
            ),
        ));
        return;
    }
    let declared: Vec<&str> = contract
        .inputs
        .iter()
        .filter_map(|(name, decl)| decl.required.then_some(name.as_str()))
        .collect();
    check_declared_map(
        BindingCheck {
            call_from: &call.from,
            caller_path,
            job_key,
            callee_path,
            field: "with",
            noun: "required input",
            action: "pass",
            declared: &declared,
        },
        raw,
        &mut out.findings,
        &mut out.unanalyzable,
    );
}
