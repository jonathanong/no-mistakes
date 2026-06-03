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

fn traverse<'a, E>(
    roots: &'a [String],
    include_deps: bool,
    include_dependents: bool,
    forward: &'a HashMap<String, Vec<E>>,
    reverse: &'a HashMap<String, Vec<E>>,
) -> Vec<E>
where
    E: RelatedEdge,
{
    // ⚡ Bolt: Track nodes using string slices (&str) instead of cloning Strings.
    // This avoids expensive heap allocations for every visited node during traversal.
    let mut seen = HashSet::new();
    let mut queue = VecDeque::new();
    for root in roots {
        seen.insert(root.as_str());
        queue.push_back(root.as_str());
    }
    let mut out = Vec::new();
    while let Some(node) = queue.pop_front() {
        if include_deps {
            if let Some(edges) = forward.get(node) {
                for edge in edges {
                    let next = edge.target();
                    if seen.insert(next) {
                        queue.push_back(next);
                    }
                    out.push(edge.clone());
                }
            }
        }
        if include_dependents {
            if let Some(edges) = reverse.get(node) {
                for edge in edges {
                    let next = edge.target();
                    if seen.insert(next) {
                        queue.push_back(next);
                    }
                    out.push(edge.clone());
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out
}
