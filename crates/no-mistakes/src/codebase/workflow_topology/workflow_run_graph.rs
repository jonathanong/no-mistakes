//! Resolves `workflow_run` subscriptions into edges, then diagnoses cycles
//! and over-long acyclic chains. Ported from `workflow-run-graph.mts`.
//!
//! Operates on `WorkflowTrigger.config`, which by this point in the
//! pipeline is already the order-preserving JSON-shaped value stored on the
//! parsed [`model::WorkflowNode`] (not raw YAML) — so this module walks
//! [`OrderedJson`], not `serde_yaml::Value` like [`super::parse`] and
//! [`super::workflow_values`] do.

use super::model;
use super::value_primitives::OrderedJson;
use cycle_diagnostics::diagnose_workflow_run_graph;
use std::collections::{HashMap, HashSet};

mod cycle_diagnostics;

pub fn resolve_workflow_run_graph(
    workflows: &[model::WorkflowNode],
    diagnostics: &mut Vec<model::WorkflowTopologyDiagnostic>,
) -> Vec<model::WorkflowTopologyEdge> {
    let mut by_name: HashMap<String, Vec<&model::WorkflowNode>> = HashMap::new();
    for workflow in workflows {
        by_name
            .entry(workflow.name.to_lowercase())
            .or_default()
            .push(workflow);
    }

    let mut edges: Vec<model::WorkflowRunEdge> = Vec::new();
    let mut resolved_pairs: HashSet<(String, String)> = HashSet::new();

    for subscriber in workflows {
        let Some(trigger) = subscriber
            .triggers
            .iter()
            .find(|trigger| trigger.event == "workflow_run")
        else {
            continue;
        };
        let Some(config @ OrderedJson::Object(_)) = &trigger.config else {
            continue;
        };
        let metadata = workflow_run_metadata(config);
        for source_name in json_string_list(config.get("workflows")) {
            let sources = by_name
                .get(&source_name.to_lowercase())
                .cloned()
                .unwrap_or_default();
            if sources.is_empty() {
                diagnostics.push(model::WorkflowTopologyDiagnostic::new(
                    model::DiagnosticCode::MissingWorkflowRunSource,
                    format!(
                        "{} subscribes to missing workflow {}",
                        subscriber.path,
                        quoted(&source_name)
                    ),
                    subscriber.path.clone(),
                ));
                continue;
            }
            if sources.len() > 1 {
                let mut paths: Vec<&str> =
                    sources.iter().map(|source| source.path.as_str()).collect();
                paths.sort();
                diagnostics.push(model::WorkflowTopologyDiagnostic::new(
                    model::DiagnosticCode::AmbiguousWorkflowRunSource,
                    format!(
                        "{} subscribes to ambiguous workflow {} across {}",
                        subscriber.path,
                        quoted(&source_name),
                        paths.join(", ")
                    ),
                    subscriber.path.clone(),
                ));
                continue;
            }
            let source = sources[0];
            if !resolved_pairs.insert((source.path.clone(), subscriber.path.clone())) {
                continue;
            }
            edges.push(model::WorkflowRunEdge {
                from: source.path.clone(),
                to: subscriber.path.clone(),
                types: metadata.types.clone(),
                branches: metadata.branches.clone(),
                branches_ignore: metadata.branches_ignore.clone(),
            });
        }
    }

    diagnose_workflow_run_graph(workflows, &edges, diagnostics);
    edges
        .into_iter()
        .map(model::WorkflowTopologyEdge::WorkflowRun)
        .collect()
}

fn quoted(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

struct WorkflowRunMetadata {
    types: Option<Vec<String>>,
    branches: Option<Vec<String>>,
    branches_ignore: Option<Vec<String>>,
}

fn workflow_run_metadata(config: &OrderedJson) -> WorkflowRunMetadata {
    let types = json_string_list(config.get("types"));
    let branches = json_string_list(config.get("branches"));
    let branches_ignore = json_string_list(config.get("branches-ignore"));
    WorkflowRunMetadata {
        types: (!types.is_empty()).then_some(types),
        branches: (!branches.is_empty()).then_some(branches),
        branches_ignore: (!branches_ignore.is_empty()).then_some(branches_ignore),
    }
}

fn json_string_list(value: Option<&OrderedJson>) -> Vec<String> {
    match value {
        Some(OrderedJson::String(text)) => vec![text.clone()],
        Some(OrderedJson::Array(items)) => items
            .iter()
            .filter_map(OrderedJson::as_str)
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}
