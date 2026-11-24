use super::super::model;
use super::ParsedWorkflow;

pub(crate) fn malformed_workflow(path: &str, message: &str) -> ParsedWorkflow {
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
        diagnostics: vec![model::WorkflowTopologyDiagnostic::new(
            model::DiagnosticCode::MalformedWorkflow,
            message.to_string(),
            path,
        )],
        output_references: Vec::new(),
    }
}

pub(super) fn workflow_basename(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_string()
}
