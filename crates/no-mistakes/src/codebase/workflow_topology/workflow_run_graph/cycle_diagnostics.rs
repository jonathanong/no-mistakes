//! `workflow_run` cycle and chain-limit diagnostics, split out of [`super`]
//! to stay under the crate's per-file line limit.

use super::super::graph_algorithms;
use super::super::model;
use std::collections::{HashMap, HashSet, VecDeque};

const MAX_WORKFLOW_RUN_CHAIN_EDGES: u32 = 3;

pub(super) fn diagnose_workflow_run_graph(
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
