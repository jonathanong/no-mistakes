//! Parses a single workflow YAML file into a [`ParsedWorkflow`] fragment
//! (node, jobs, edges, diagnostics, output references), ported from
//! `parse-workflow.mts`. [`super::mod`]'s loader parses every workflow file
//! this way, then merges and cross-references the fragments.

use super::case_insensitive_lookup::{CaseInsensitiveLookup, Resolution};
use super::expression_references;
use super::model;
use super::value_primitives;
use super::workflow_values;
use serde_yaml::Value;
use std::path::Path;

/// A `needs.<callJobKey>.outputs.<output>` reference found anywhere in a
/// job (its own scalars, its `if:`, or a step's `if:`), keyed by the
/// consuming job so [`super::call_contract_diagnostics`] can validate it
/// against the referenced call's callee contract.
pub struct ParsedWorkflowOutputReference {
    pub consumer_job_id: String,
    pub call_job_key: String,
    pub output: String,
}

pub struct ParsedWorkflow {
    pub node: model::WorkflowNode,
    pub jobs: Vec<model::WorkflowJobNode>,
    pub edges: Vec<model::WorkflowTopologyEdge>,
    pub diagnostics: Vec<model::WorkflowTopologyDiagnostic>,
    pub output_references: Vec<ParsedWorkflowOutputReference>,
}

/// Reads and parses one workflow YAML file. Both I/O failures and YAML
/// parse errors become a single `malformed-workflow` diagnostic, matching
/// the TS engine's `parseWorkflowFile` (which does its own file read and
/// doesn't distinguish the two failure modes) — an intentional divergence
/// from the sibling `ci_graph::WorkflowSet::load`, which does distinguish
/// them. Note the diagnostic *message text* necessarily differs from the
/// TS engine's: `serde_yaml`'s and `js-yaml`'s error messages are
/// different libraries' wording for the same failure, not a
/// reproducible string.
pub fn parse_workflow_file(root_dir: &Path, path: &str) -> ParsedWorkflow {
    let content = match std::fs::read_to_string(root_dir.join(path)) {
        Ok(content) => content,
        Err(error) => return malformed_result(path, &error.to_string()),
    };
    let value: Value = match serde_yaml::from_str(&content) {
        Ok(value) => value,
        Err(error) => return malformed_result(path, &error.to_string()),
    };
    if !value_primitives::is_record(Some(&value)) {
        return malformed_result(path, "workflow root must be a mapping");
    }

    let triggers = workflow_values::parse_triggers(value.get("on"));
    let job_entries: Vec<(String, &Value)> = match value.get("jobs") {
        Some(Value::Mapping(mapping)) => mapping
            .iter()
            .filter_map(|(key, value)| Some((workflow_values::key_name(key)?, value)))
            .collect(),
        _ => Vec::new(),
    };

    let mut jobs = Vec::new();
    let mut edges = Vec::new();
    let mut diagnostics = Vec::new();
    let mut output_references = Vec::new();
    let job_keys =
        CaseInsensitiveLookup::new(job_entries.iter().map(|(key, value)| (key.clone(), *value)));

    for (key, unknown_job) in job_entries.iter().map(|(key, value)| (key, *value)) {
        if !value_primitives::is_record(Some(unknown_job)) {
            continue;
        }
        let job_id = format!("{path}#{key}");
        let matrix = workflow_values::matrix_from_job(unknown_job);
        let concurrency = workflow_values::parse_concurrency(unknown_job.get("concurrency"));
        jobs.push(model::WorkflowJobNode {
            id: job_id.clone(),
            workflow_id: path.to_string(),
            key: key.clone(),
            kind: if matrix.is_none() {
                model::JobKind::Job
            } else {
                model::JobKind::MatrixTemplate
            },
            name: value_primitives::string_value(unknown_job.get("name")),
            condition: value_primitives::string_value(unknown_job.get("if")),
            matrix,
            concurrency,
            steps: workflow_values::parse_steps(unknown_job.get("steps")),
        });
        for reference in expression_references::workflow_output_references(unknown_job) {
            output_references.push(ParsedWorkflowOutputReference {
                consumer_job_id: job_id.clone(),
                call_job_key: reference.call_job_id,
                output: reference.output,
            });
        }
        add_job_edges(
            path,
            &job_id,
            unknown_job,
            &job_keys,
            &mut edges,
            &mut diagnostics,
        );
    }

    let concurrency = workflow_values::parse_concurrency(value.get("concurrency"));
    let workflow_call = workflow_values::parse_workflow_call(value.get("on"));
    let mut job_ids: Vec<String> = jobs.iter().map(|job| job.id.clone()).collect();
    job_ids.sort();

    ParsedWorkflow {
        node: model::WorkflowNode {
            id: path.to_string(),
            path: path.to_string(),
            name: value_primitives::string_value(value.get("name"))
                .unwrap_or_else(|| workflow_basename(path)),
            callable: workflow_call.is_some(),
            workflow_call,
            triggers,
            job_ids,
            concurrency,
        },
        jobs,
        edges,
        diagnostics,
        output_references,
    }
}

fn add_job_edges(
    path: &str,
    job_id: &str,
    job: &Value,
    job_keys: &CaseInsensitiveLookup<&Value>,
    edges: &mut Vec<model::WorkflowTopologyEdge>,
    diagnostics: &mut Vec<model::WorkflowTopologyDiagnostic>,
) {
    for need in value_primitives::string_list(job.get("needs")) {
        let (resolved_key, is_missing) = match job_keys.resolve(&need) {
            Resolution::Ambiguous => continue,
            Resolution::Resolved { key, .. } => (key.to_string(), false),
            Resolution::Missing => (need.clone(), true),
        };
        let target = format!("{path}#{resolved_key}");
        edges.push(model::WorkflowTopologyEdge::Needs(model::NeedsEdge {
            from: target.clone(),
            to: job_id.to_string(),
        }));
        if is_missing {
            diagnostics.push(
                model::WorkflowTopologyDiagnostic::new(
                    model::DiagnosticCode::MissingNeedsDependency,
                    format!("{job_id} needs missing job {target}"),
                    path,
                )
                .with_job(job_id),
            );
        }
    }
    if let Some(uses) = value_primitives::string_value(job.get("uses")) {
        edges.push(model::WorkflowTopologyEdge::Calls(
            workflow_values::call_edge(job_id, &uses, job),
        ));
    }
}

fn malformed_result(path: &str, message: &str) -> ParsedWorkflow {
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

fn workflow_basename(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_string()
}
