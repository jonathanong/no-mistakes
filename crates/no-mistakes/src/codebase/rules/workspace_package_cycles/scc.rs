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
            cycles.extend(component_cycles(graph, &component));
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

fn component_cycles(
    graph: &BTreeMap<String, BTreeSet<String>>,
    component: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut cycles = BTreeSet::new();
    for from in component {
        for to in graph.get(from).into_iter().flatten() {
            if !component.contains(to) {
                continue;
            }
            if let Some(path) = path_to(to, from, graph, component) {
                let mut cycle = vec![from.clone()];
                cycle.extend(path);
                cycles.insert(canonical_cycle(&cycle.join(" -> ")));
            }
        }
    }
    cycles
}

fn path_to(
    current: &str,
    target: &str,
    graph: &BTreeMap<String, BTreeSet<String>>,
    component: &BTreeSet<String>,
) -> Option<Vec<String>> {
    let mut stack = vec![(current.to_string(), vec![current.to_string()])];
    let mut seen = BTreeSet::new();
    seen.insert(current.to_string());
    while let Some((node, path)) = stack.pop() {
        if node == target {
            return Some(path);
        }
        let mut next_nodes = graph
            .get(&node)
            .into_iter()
            .flatten()
            .filter(|next| component.contains(*next) && !seen.contains(*next))
            .cloned()
            .collect::<Vec<_>>();
        next_nodes.reverse();
        for next in next_nodes {
            let mut next_path = path.clone();
            next_path.push(next.clone());
            seen.insert(next.clone());
            stack.push((next, next_path));
        }
    }
    None
}
