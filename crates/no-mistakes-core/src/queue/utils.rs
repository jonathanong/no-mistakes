use std::collections::{HashMap, HashSet, VecDeque};

pub(crate) trait RelatedEdge: Clone + Eq + Ord {
    fn source(&self) -> &str;
    fn target(&self) -> &str;
    fn reversed(&self) -> Self;
}

pub(crate) fn related_from_edges<E>(
    edges: &[E],
    roots: &[String],
    include_deps: bool,
    include_dependents: bool,
) -> Vec<E>
where
    E: RelatedEdge,
{
    let mut forward: HashMap<String, Vec<E>> = HashMap::new();
    let mut reverse: HashMap<String, Vec<E>> = HashMap::new();
    for edge in edges {
        forward
            .entry(edge.source().to_owned())
            .or_default()
            .push(edge.clone());
        reverse
            .entry(edge.target().to_owned())
            .or_default()
            .push(edge.reversed());
    }
    traverse(roots, include_deps, include_dependents, &forward, &reverse)
}

fn traverse<E>(
    roots: &[String],
    include_deps: bool,
    include_dependents: bool,
    forward: &HashMap<String, Vec<E>>,
    reverse: &HashMap<String, Vec<E>>,
) -> Vec<E>
where
    E: RelatedEdge,
{
    let mut seen = HashSet::new();
    let mut queue = VecDeque::new();
    for root in roots {
        seen.insert(root.clone());
        queue.push_back(root.clone());
    }
    let mut out = Vec::new();
    while let Some(node) = queue.pop_front() {
        if include_deps {
            for edge in forward.get(&node).into_iter().flatten() {
                if seen.insert(edge.target().to_owned()) {
                    queue.push_back(edge.target().to_owned());
                }
                out.push(edge.clone());
            }
        }
        if include_dependents {
            for edge in reverse.get(&node).into_iter().flatten() {
                if seen.insert(edge.source().to_owned()) {
                    queue.push_back(edge.source().to_owned());
                }
                out.push(edge.clone());
            }
        }
    }
    out.sort();
    out.dedup();
    out
}
