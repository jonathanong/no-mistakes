//! Validates reusable-workflow calls against their callee's declared
//! `workflow_call` contract (inputs, secrets, outputs), ported from
//! `call-contract-diagnostics.mts`. Only local calls resolving to a
//! callable workflow are checked — remote and non-callable targets are
//! opaque (already diagnosed elsewhere by [`super::topology_graph`]).

use super::case_insensitive_lookup::{CaseInsensitiveLookup, Resolution};
use super::model;
use super::parse::ParsedWorkflowOutputReference;
use super::topology_identifiers;
use secrets_outputs::{diagnose_outputs, diagnose_secrets};
use std::collections::HashMap;

mod secrets_outputs;

pub fn diagnose_workflow_call_contracts(
    workflows: &[model::WorkflowNode],
    jobs: &[model::WorkflowJobNode],
    edges: &[model::WorkflowTopologyEdge],
    output_references: &[ParsedWorkflowOutputReference],
    diagnostics: &mut Vec<model::WorkflowTopologyDiagnostic>,
) {
    let workflows_by_path: HashMap<String, &model::WorkflowNode> = workflows
        .iter()
        .map(|workflow| (workflow.path.clone(), workflow))
        .collect();
    let call_edges: Vec<&model::WorkflowCallEdge> = edges
        .iter()
        .filter_map(|edge| match edge {
            model::WorkflowTopologyEdge::Calls(call) => Some(call),
            _ => None,
        })
        .collect();

    for edge in &call_edges {
        let Some(callee) =
            resolved_callable_callee(edge.local, edge.to.as_deref(), &workflows_by_path)
        else {
            continue;
        };
        diagnose_inputs(edge, callee, diagnostics);
        diagnose_secrets(edge, callee, diagnostics);
    }
    diagnose_outputs(
        jobs,
        &call_edges,
        output_references,
        &workflows_by_path,
        diagnostics,
    );
}

fn resolved_callable_callee<'a>(
    local: bool,
    to: Option<&str>,
    workflows_by_path: &'a HashMap<String, &'a model::WorkflowNode>,
) -> Option<&'a model::WorkflowNode> {
    if !local {
        return None;
    }
    let callee = *workflows_by_path.get(to?)?;
    callee.callable.then_some(callee)
}

fn diagnose_inputs(
    edge: &model::WorkflowCallEdge,
    callee: &model::WorkflowNode,
    diagnostics: &mut Vec<model::WorkflowTopologyDiagnostic>,
) {
    let Some(contract) = &callee.workflow_call else {
        return;
    };
    let declarations = CaseInsensitiveLookup::new(
        contract
            .inputs
            .iter()
            .map(|(key, value)| (key.clone(), value)),
    );
    let bindings = CaseInsensitiveLookup::new(
        edge.bindings
            .inputs
            .iter()
            .map(|(key, value)| (key.clone(), value)),
    );

    for (name, declaration) in declarations.unique_entries() {
        match bindings.resolve(name) {
            Resolution::Ambiguous => continue,
            Resolution::Missing => {
                if declaration.required {
                    diagnostics.push(call_diagnostic(
                        edge,
                        &callee.path,
                        model::DiagnosticCode::MissingWorkflowCallInput,
                        format!(
                            "{} does not provide required input {name} to {}",
                            edge.from, callee.path
                        ),
                    ));
                }
            }
            Resolution::Resolved { key, value } => {
                if !matches_declared_type(value, declaration) {
                    diagnostics.push(call_diagnostic(
                        edge,
                        &callee.path,
                        model::DiagnosticCode::WorkflowCallInputTypeMismatch,
                        format!(
                            "{} provides input {key} as {}, but {} declares {}",
                            edge.from,
                            json_scalar_type_name(value),
                            callee.path,
                            declaration
                                .input_type
                                .map(model::WorkflowCallInputType::as_str)
                                .unwrap_or("undefined"),
                        ),
                    ));
                }
            }
        }
    }
    for (name, _declaration) in bindings.unique_entries() {
        if !matches!(declarations.resolve(name), Resolution::Missing) {
            continue;
        }
        diagnostics.push(call_diagnostic(
            edge,
            &callee.path,
            model::DiagnosticCode::UnknownWorkflowCallInput,
            format!(
                "{} provides unknown input {name} to {}",
                edge.from, callee.path
            ),
        ));
    }
}

fn call_diagnostic(
    edge: &model::WorkflowCallEdge,
    callee_workflow_path: &str,
    code: model::DiagnosticCode,
    message: String,
) -> model::WorkflowTopologyDiagnostic {
    model::WorkflowTopologyDiagnostic::new(
        code,
        message,
        topology_identifiers::workflow_path_from_id(&edge.from),
    )
    .with_job(&edge.from)
    .with_call_job(&edge.from)
    .with_callee(callee_workflow_path)
}

/// An opaque (`${{ ... }}`-containing) literal binding is never checked; an
/// absent declared type accepts anything.
fn matches_declared_type(
    value: &model::JsonScalar,
    declaration: &model::WorkflowCallInput,
) -> bool {
    let Some(declared_type) = declaration.input_type else {
        return true;
    };
    if let model::JsonScalar::Text(text) = value {
        if text.contains("${{") {
            return true;
        }
    }
    matches!(
        (value, declared_type),
        (
            model::JsonScalar::Bool(_),
            model::WorkflowCallInputType::Boolean
        ) | (
            model::JsonScalar::Number(_),
            model::WorkflowCallInputType::Number
        ) | (
            model::JsonScalar::Text(_),
            model::WorkflowCallInputType::String
        )
    )
}

fn json_scalar_type_name(value: &model::JsonScalar) -> &'static str {
    match value {
        model::JsonScalar::Bool(_) => "boolean",
        model::JsonScalar::Number(_) => "number",
        model::JsonScalar::Text(_) => "string",
    }
}
