//! Edge/diagnostic dedup keys and same-run reachability memoization, split
//! out of [`super`] to stay under the crate's per-file line limit.
//! Re-exported by [`super`] so every `artifact_resolution_helpers::edge_key`
//! -style path elsewhere in the crate keeps working unchanged.

use super::super::artifact_resolution_types::ArtifactRunContext;
use super::super::artifact_types::ArtifactEdge;
use super::super::model::WorkflowTopologyDiagnostic;
use std::collections::HashMap;

/// Deduplicates by [`edge_key`], keeping the *last* edge for each key but
/// the position of its *first* occurrence — matching `new
/// Map(edges.map(...)).values()`'s semantics (re-setting an existing Map
/// key updates its value without moving it).
pub fn unique_edges(edges: &[ArtifactEdge]) -> Vec<ArtifactEdge> {
    let mut order: Vec<String> = Vec::new();
    let mut by_key: HashMap<String, ArtifactEdge> = HashMap::new();
    for edge in edges {
        let key = edge_key(edge);
        if !by_key.contains_key(&key) {
            order.push(key.clone());
        }
        by_key.insert(key, edge.clone());
    }
    order
        .into_iter()
        .map(|key| by_key.remove(&key).expect("just inserted above"))
        .collect()
}

pub fn edge_set_key(edges: &[ArtifactEdge]) -> String {
    let mut keys: Vec<String> = unique_edges(edges).iter().map(edge_key).collect();
    keys.sort();
    keys.join("|")
}

pub fn edge_key(edge: &ArtifactEdge) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}",
        edge.from,
        edge.producer_step,
        edge.to,
        edge.consumer_step,
        edge.name,
        edge.match_kind.as_str()
    )
}

/// A stable dedup key for a diagnostic, mirroring the TS engine's
/// `` `${code}|${jobId}|${message}` `` template literal — where an absent
/// `jobId` renders as the literal string `"undefined"`, not empty (both
/// diagnostic constructors here always set `jobId`, so that branch is
/// unreachable in practice, but the fallback stays faithful).
pub fn diagnostic_key(diagnostic: Option<&WorkflowTopologyDiagnostic>) -> String {
    match diagnostic {
        Some(diagnostic) => format!(
            "{}|{}|{}",
            diagnostic.code.as_str(),
            diagnostic.job_id.as_deref().unwrap_or("undefined"),
            diagnostic.message,
        ),
        None => String::new(),
    }
}

/// Whether `to` is reachable from `from` via `context.adjacency`
/// (`needs`-derived same-run precedence arcs), memoizing the full reachable
/// set per `from` on first query.
pub fn occurrence_reaches(context: &ArtifactRunContext, from: &str, to: &str) -> bool {
    if let Some(cached) = context.reachability_cache.borrow().get(from) {
        return cached.contains(to);
    }
    let mut pending: Vec<String> = context
        .adjacency
        .get(from)
        .map(|set| set.iter().cloned().collect())
        .unwrap_or_default();
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
    while let Some(current) = pending.pop() {
        if !visited.insert(current.clone()) {
            continue;
        }
        if let Some(next) = context.adjacency.get(&current) {
            pending.extend(next.iter().cloned());
        }
    }
    let reaches_to = visited.contains(to);
    context
        .reachability_cache
        .borrow_mut()
        .insert(from.to_string(), visited);
    reaches_to
}
