//! Tarjan's strongly-connected-components algorithm, ported from
//! `graph-algorithms.mts`. Deterministic: neighbor traversal order and each
//! output component are both sorted, so the same graph always yields the
//! same component list in the same order — load-bearing for reproducing
//! stable `job-dependency-cycle` / `workflow-run-cycle` diagnostics.

use std::collections::{HashMap, HashSet};

struct State {
    next_index: usize,
    indexes: HashMap<String, usize>,
    low_links: HashMap<String, usize>,
    stack: Vec<String>,
    on_stack: HashSet<String>,
    components: Vec<Vec<String>>,
}

/// Finds every strongly connected component of `adjacency` (node → direct
/// successors). A node that is only ever a target (never a key) is still
/// visited, with an implicit empty successor set — callers of this module
/// always pre-populate every node as a key, but the algorithm doesn't
/// depend on that.
pub fn strongly_connected_components(
    adjacency: &HashMap<String, HashSet<String>>,
) -> Vec<Vec<String>> {
    let mut nodes: Vec<&String> = adjacency.keys().collect();
    nodes.sort();

    let mut state = State {
        next_index: 0,
        indexes: HashMap::new(),
        low_links: HashMap::new(),
        stack: Vec::new(),
        on_stack: HashSet::new(),
        components: Vec::new(),
    };
    for node in nodes {
        if !state.indexes.contains_key(node) {
            visit(node, adjacency, &mut state);
        }
    }
    state.components
}

fn visit(node: &str, adjacency: &HashMap<String, HashSet<String>>, state: &mut State) {
    state.indexes.insert(node.to_string(), state.next_index);
    state.low_links.insert(node.to_string(), state.next_index);
    state.next_index += 1;
    state.stack.push(node.to_string());
    state.on_stack.insert(node.to_string());

    let mut targets: Vec<String> = adjacency
        .get(node)
        .map(|set| set.iter().cloned().collect())
        .unwrap_or_default();
    targets.sort();

    for target in &targets {
        if !state.indexes.contains_key(target) {
            visit(target, adjacency, state);
            let merged = state.low_links[node].min(state.low_links[target]);
            state.low_links.insert(node.to_string(), merged);
        } else if state.on_stack.contains(target) {
            let merged = state.low_links[node].min(state.indexes[target]);
            state.low_links.insert(node.to_string(), merged);
        }
    }

    if state.low_links[node] != state.indexes[node] {
        return;
    }
    let mut component = Vec::new();
    loop {
        let member = state
            .stack
            .pop()
            .expect("node is on the stack until popped");
        state.on_stack.remove(&member);
        let is_node = member == node;
        component.push(member);
        if is_node {
            break;
        }
    }
    component.sort();
    state.components.push(component);
}
