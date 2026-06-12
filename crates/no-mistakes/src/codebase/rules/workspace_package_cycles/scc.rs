use super::canonical_cycle;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn cycle_keys(graph: &BTreeMap<String, BTreeSet<String>>) -> BTreeSet<String> {
    let reachable = reachability(graph);
    let mut assigned = BTreeSet::new();
    let mut cycles = BTreeSet::new();
    for node in graph.keys() {
        if assigned.contains(node) {
            continue;
        }
        let component: BTreeSet<String> = graph
            .keys()
            .filter(|other| {
                reachable
                    .get(node)
                    .is_some_and(|nodes| nodes.contains(*other))
                    && reachable
                        .get(*other)
                        .is_some_and(|nodes| nodes.contains(node))
            })
            .cloned()
            .collect();
        assigned.extend(component.iter().cloned());
        if component.len() > 1 {
            if let Some(cycle) = first_cycle(node, node, graph, &component, &mut Vec::new()) {
                cycles.insert(canonical_cycle(&cycle.join(" -> ")));
            }
        } else if graph.get(node).is_some_and(|deps| deps.contains(node)) {
            cycles.insert(canonical_cycle(&format!("{node} -> {node}")));
        }
    }
    cycles
}

fn reachability(graph: &BTreeMap<String, BTreeSet<String>>) -> BTreeMap<String, BTreeSet<String>> {
    graph
        .keys()
        .map(|node| (node.clone(), reachable_from(node, graph)))
        .collect()
}

fn reachable_from(start: &str, graph: &BTreeMap<String, BTreeSet<String>>) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut stack = vec![start.to_string()];
    while let Some(node) = stack.pop() {
        if !seen.insert(node.clone()) {
            continue;
        }
        stack.extend(graph.get(&node).into_iter().flatten().cloned());
    }
    seen
}

fn first_cycle(
    start: &str,
    current: &str,
    graph: &BTreeMap<String, BTreeSet<String>>,
    component: &BTreeSet<String>,
    stack: &mut Vec<String>,
) -> Option<Vec<String>> {
    stack.push(current.to_string());
    for next in graph.get(current).into_iter().flatten() {
        if !component.contains(next) {
            continue;
        }
        if next == start {
            let mut cycle = stack.clone();
            cycle.push(start.to_string());
            stack.pop();
            return Some(cycle);
        }
        if !stack.iter().any(|seen| seen == next) {
            if let Some(cycle) = first_cycle(start, next, graph, component, stack) {
                stack.pop();
                return Some(cycle);
            }
        }
    }
    stack.pop();
    None
}
