//! Shortest load path from a test or setup root to an unreadable module.
use crate::codebase::dependencies::graph::{DepGraph, NodeId};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::relative_slash_path;
use crate::fx::FxHashMap;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};

pub(super) fn reach_path(
    graph: &DepGraph,
    gate_roots: &[PathBuf],
    target: &Path,
    root: &Path,
) -> String {
    let kinds = super::super::reachable::load_kinds();
    let target = normalize_path(target);
    let mut parent: FxHashMap<PathBuf, Option<PathBuf>> = FxHashMap::default();
    let mut queue = VecDeque::new();
    for gate in gate_roots {
        let gate = normalize_path(gate);
        if parent.contains_key(&gate) {
            continue;
        }
        parent.insert(gate.clone(), None);
        queue.push_back(gate);
    }
    while let Some(file) = queue.pop_front() {
        if file == target {
            break;
        }
        let Some(neighbors) = graph.dependencies_of_node(&NodeId::file(&file)) else {
            continue;
        };
        for (node, kind) in neighbors {
            if !kinds.contains(kind) {
                continue;
            }
            let Some(next) = node.as_file() else {
                continue;
            };
            let next = normalize_path(next);
            if parent.contains_key(&next) {
                continue;
            }
            parent.insert(next.clone(), Some(file.clone()));
            queue.push_back(next);
        }
    }
    if !parent.contains_key(&target) {
        return String::new();
    }
    let mut chain = Vec::new();
    let mut cursor = target;
    loop {
        chain.push(relative_slash_path(root, &cursor));
        match parent.get(&cursor) {
            Some(Some(previous)) => cursor = previous.clone(),
            _ => break,
        }
    }
    chain.reverse();
    chain.join(" → ")
}

#[cfg(test)]
#[path = "closure_path/tests.rs"]
mod tests;
