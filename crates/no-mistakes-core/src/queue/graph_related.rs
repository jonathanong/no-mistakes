use crate::queue::graph::RelatedDirection;
use crate::queue::graph_model::ProjectReport;
use crate::queue::types::Edge;
use std::collections::{HashMap, HashSet, VecDeque};

pub fn related(report: &ProjectReport, roots: &[String], direction: RelatedDirection) -> Vec<Edge> {
    let mut forward: HashMap<&str, Vec<&Edge>> = HashMap::new();
    let mut reverse: HashMap<&str, Vec<&Edge>> = HashMap::new();
    for edge in &report.edges {
        if matches!(direction, RelatedDirection::Deps | RelatedDirection::Both) {
            forward.entry(edge.from.as_str()).or_default().push(edge);
        }
        if matches!(
            direction,
            RelatedDirection::Dependents | RelatedDirection::Both
        ) {
            reverse.entry(edge.to.as_str()).or_default().push(edge);
        }
    }
    traverse(roots, direction, &forward, &reverse)
}

fn traverse(
    roots: &[String],
    direction: RelatedDirection,
    forward: &HashMap<&str, Vec<&Edge>>,
    reverse: &HashMap<&str, Vec<&Edge>>,
) -> Vec<Edge> {
    let mut seen = HashSet::new();
    let mut queue = VecDeque::new();
    for root in roots {
        seen.insert(root.as_str());
        queue.push_back(root.as_str());
    }
    let mut out = Vec::new();
    while let Some(node) = queue.pop_front() {
        if matches!(direction, RelatedDirection::Deps | RelatedDirection::Both) {
            if let Some(edges) = forward.get(node) {
                for edge in edges {
                    if seen.insert(edge.to.as_str()) {
                        queue.push_back(edge.to.as_str());
                    }
                    out.push((*edge).clone());
                }
            }
        }
        if matches!(
            direction,
            RelatedDirection::Dependents | RelatedDirection::Both
        ) {
            if let Some(edges) = reverse.get(node) {
                for edge in edges {
                    if seen.insert(edge.from.as_str()) {
                        queue.push_back(edge.from.as_str());
                    }
                    out.push(Edge {
                        from: edge.to.clone(),
                        to: edge.from.clone(),
                        kind: edge.kind,
                    });
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out
}
