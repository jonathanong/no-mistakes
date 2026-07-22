//! Resolves `workflow_run` subscriptions into edges, then diagnoses cycles
//! and over-long acyclic chains. Ported from `workflow-run-graph.mts`.
//!
//! Operates on `WorkflowTrigger.config`, which by this point in the
//! pipeline is already the order-preserving JSON-shaped value stored on the
//! parsed [`model::WorkflowNode`] (not raw YAML) — so this module walks
//! [`OrderedJson`], not `serde_yaml::Value` like [`super::parse`] and
//! [`super::workflow_values`] do.

use super::graph_algorithms;
use super::model;
use super::value_primitives::OrderedJson;
use std::collections::{HashMap, HashSet, VecDeque};

const MAX_WORKFLOW_RUN_CHAIN_EDGES: u32 = 3;

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

fn diagnose_workflow_run_graph(
    workflows: &[model::WorkflowNode],
    edges: &[model::WorkflowRunEdge],
    diagnostics: &mut Vec<model::WorkflowTopologyDiagnostic>,
) {
    let mut adjacency: HashMap<String, HashSet<String>> = workflows
        .iter()
        .map(|workflow| (workflow.path.clone(), HashSet::new()))
        .collect();
    for edge in edges {
        adjacency
            .get_mut(&edge.from)
            .expect("workflow-run edge.from is always a known workflow path")
            .insert(edge.to.clone());
    }

    let mut cyclic_nodes: HashSet<String> = HashSet::new();
    for component in graph_algorithms::strongly_connected_components(&adjacency) {
        let self_cycle = component.len() == 1
            && adjacency
                .get(&component[0])
                .is_some_and(|set| set.contains(&component[0]));
        if component.len() < 2 && !self_cycle {
            continue;
        }
        cyclic_nodes.extend(component.iter().cloned());
        let witness = cycle_witness(&component, &adjacency);
        diagnostics.push(model::WorkflowTopologyDiagnostic::new(
            model::DiagnosticCode::WorkflowRunCycle,
            format!("workflow_run cycle: {}", witness.join(" -> ")),
            witness[0].clone(),
        ));
    }
    diagnose_chain_limits(&adjacency, &cyclic_nodes, diagnostics);
}

/// Finds a deterministic (sorted-DFS, shortest-first) cycle path through
/// `component`, starting and ending at its lexicographically-first member.
fn cycle_witness(
    component: &[String],
    adjacency: &HashMap<String, HashSet<String>>,
) -> Vec<String> {
    let start = &component[0];
    let members: HashSet<&String> = component.iter().collect();
    if adjacency.get(start).is_some_and(|set| set.contains(start)) {
        return vec![start.clone(), start.clone()];
    }
    let mut witness: Vec<String> = Vec::new();
    let mut visited: HashSet<String> = HashSet::from([start.clone()]);
    search_cycle_witness(
        start,
        start,
        std::slice::from_ref(start),
        &mut visited,
        &members,
        adjacency,
        &mut witness,
    );
    witness
}

#[allow(clippy::too_many_arguments)]
fn search_cycle_witness(
    start: &str,
    current: &str,
    path: &[String],
    visited: &mut HashSet<String>,
    members: &HashSet<&String>,
    adjacency: &HashMap<String, HashSet<String>>,
    witness: &mut Vec<String>,
) -> bool {
    let Some(targets) = adjacency.get(current) else {
        return false;
    };
    let mut sorted_targets: Vec<&String> = targets.iter().collect();
    sorted_targets.sort();
    for target in sorted_targets {
        if !members.contains(target) {
            continue;
        }
        if target == start {
            let mut result = path.to_vec();
            result.push(start.to_string());
            *witness = result;
            return true;
        }
        if visited.contains(target) {
            continue;
        }
        visited.insert(target.clone());
        let mut next_path = path.to_vec();
        next_path.push(target.clone());
        if search_cycle_witness(
            start, target, &next_path, visited, members, adjacency, witness,
        ) {
            return true;
        }
        visited.remove(target);
    }
    false
}

/// Kahn's algorithm over the acyclic subset of `adjacency`, tracking the
/// longest path reaching each node from any zero-indegree root (ties
/// broken by the lexicographically smaller `\0`-joined path) — the
/// deterministic witness for `workflow-run-chain-limit`.
fn diagnose_chain_limits(
    adjacency: &HashMap<String, HashSet<String>>,
    cyclic_nodes: &HashSet<String>,
    diagnostics: &mut Vec<model::WorkflowTopologyDiagnostic>,
) {
    let mut acyclic_nodes: Vec<String> = adjacency
        .keys()
        .filter(|node| !cyclic_nodes.contains(*node))
        .cloned()
        .collect();
    acyclic_nodes.sort();

    let mut indegree: HashMap<String, u32> =
        acyclic_nodes.iter().map(|node| (node.clone(), 0)).collect();
    for source in &acyclic_nodes {
        if let Some(targets) = adjacency.get(source) {
            for target in targets {
                if let Some(count) = indegree.get_mut(target) {
                    *count += 1;
                }
            }
        }
    }

    let mut pending: VecDeque<String> = acyclic_nodes
        .iter()
        .filter(|node| indegree[*node] == 0)
        .cloned()
        .collect();
    let mut paths: HashMap<String, Vec<String>> = pending
        .iter()
        .map(|node| (node.clone(), vec![node.clone()]))
        .collect();

    while let Some(source) = pending.pop_front() {
        let Some(targets) = adjacency.get(&source) else {
            continue;
        };
        let mut sorted_targets: Vec<&String> = targets.iter().collect();
        sorted_targets.sort();
        for target in sorted_targets {
            if !indegree.contains_key(target) {
                continue;
            }
            let mut candidate = paths.get(&source).cloned().unwrap_or_default();
            candidate.push(target.clone());
            let should_replace = match paths.get(target) {
                None => true,
                Some(current) => {
                    candidate.len() > current.len()
                        || (candidate.len() == current.len()
                            && path_key(&candidate) < path_key(current))
                }
            };
            if should_replace {
                paths.insert(target.clone(), candidate);
            }
            let remaining = indegree[target] - 1;
            indegree.insert(target.clone(), remaining);
            if remaining == 0 {
                pending.push_back(target.clone());
            }
        }
    }

    let mut sorted_paths: Vec<(&String, &Vec<String>)> = paths.iter().collect();
    sorted_paths.sort_by_key(|(left, _)| *left);
    for (subscriber, path) in sorted_paths {
        if path.len().saturating_sub(1) <= MAX_WORKFLOW_RUN_CHAIN_EDGES as usize {
            continue;
        }
        diagnostics.push(model::WorkflowTopologyDiagnostic::new(
            model::DiagnosticCode::WorkflowRunChainLimit,
            format!(
                "workflow_run chain exceeds {MAX_WORKFLOW_RUN_CHAIN_EDGES} levels: {}",
                path.join(" -> ")
            ),
            subscriber.clone(),
        ));
    }
}

fn path_key(path: &[String]) -> String {
    path.join("\0")
}
