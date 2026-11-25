use super::super::model;
use super::ParsedWorkflow;

pub(crate) fn malformed_workflow(path: &str, message: &str) -> ParsedWorkflow {
    unavailable_workflow(path, model::DiagnosticCode::MalformedWorkflow, message)
}

fn unavailable_workflow(path: &str, code: model::DiagnosticCode, message: &str) -> ParsedWorkflow {
    ParsedWorkflow {
        node: model::WorkflowNode {
            id: path.to_string(),
            path: path.to_string(),
            name: workflow_basename(path),
            callable: false,
            workflow_call: None,
            triggers: Vec::new(),
            job_ids: Vec::new(),
            concurrency: None,
            env: None,
            secret_references: None,
        },
        jobs: Vec::new(),
        edges: Vec::new(),
        diagnostics: vec![model::WorkflowTopologyDiagnostic::new(code, message, path)],
        output_references: Vec::new(),
    }
}

pub(super) fn workflow_basename(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_string()
}

pub(super) fn unlocated_workflow(
    path: &str,
    message: &str,
    value: &serde_yaml::Value,
) -> ParsedWorkflow {
    let mut workflow =
        unavailable_workflow(path, model::DiagnosticCode::UnlocatedYamlMerge, message);
    workflow.node.triggers = super::workflow_values::parse_triggers(value.get("on"));
    workflow
        .node
        .triggers
        .retain(|trigger| trigger.event != "<<");
    workflow
}
