use crate::edge_index::{CanonicalEdge, EdgeDirection, EdgeIndex};
pub trait TraversableEdge: Clone {
    type Kind: Ord + Clone;

    fn source(&self) -> &str;
    fn target(&self) -> &str;
    fn kind(&self) -> Self::Kind;

    fn identity(&self) -> (String, String, Self::Kind) {
        (
            self.source().to_string(),
            self.target().to_string(),
            self.kind(),
        )
    }
}

pub(crate) trait IndexedTraversableEdge: TraversableEdge
where
    Self::Kind: std::hash::Hash,
{
    fn reversed(&self) -> Self;
}

/// Request-scoped adapter from public string edge DTOs to the shared typed index.
pub(crate) struct EdgeTraversalIndex<E>
where
    E: IndexedTraversableEdge,
    E::Kind: std::hash::Hash,
{
    index: EdgeIndex<String, E::Kind>,
    edges: std::collections::HashMap<(String, String, E::Kind), E>,
}

impl<E> EdgeTraversalIndex<E>
where
    E: IndexedTraversableEdge,
    E::Kind: std::hash::Hash,
{
    pub(crate) fn new(edges: &[E]) -> Self {
        let mut public_edges = std::collections::HashMap::new();
        let canonical = edges.iter().map(|edge| {
            public_edges
                .entry(edge.identity())
                .or_insert_with(|| edge.clone());
            CanonicalEdge::new(
                edge.source().to_owned(),
                edge.target().to_owned(),
                edge.kind(),
            )
        });
        Self {
            index: EdgeIndex::from_edges(canonical),
            edges: public_edges,
        }
    }

    pub(crate) fn traverse(
        &self,
        roots: &[String],
        direction: EdgeDirection,
        depth: Option<usize>,
    ) -> Vec<E> {
        self.index
            .traverse(roots, direction, depth)
            .into_iter()
            .filter_map(|edge| {
                let direct = (edge.from.clone(), edge.to.clone(), edge.kind.clone());
                if let Some(public) = self.edges.get(&direct) {
                    return Some(public.clone());
                }
                self.edges
                    .get(&(edge.to, edge.from, edge.kind))
                    .map(IndexedTraversableEdge::reversed)
            })
            .collect()
    }
}

impl<E> EdgeTraversalIndex<E>
where
    E: IndexedTraversableEdge + Ord,
    E::Kind: std::hash::Hash,
{
    pub(crate) fn related(
        &self,
        roots: &[String],
        include_dependencies: bool,
        include_dependents: bool,
    ) -> Vec<E> {
        let direction = match (include_dependencies, include_dependents) {
            (true, false) => EdgeDirection::Dependencies,
            (false, true) => EdgeDirection::Dependents,
            (true, true) => EdgeDirection::Both,
            (false, false) => return Vec::new(),
        };
        let mut edges = self.traverse(roots, direction, None);
        edges.sort();
        edges.dedup();
        edges
    }
}

pub fn edge_view<E: TraversableEdge>(
    all_edges: &[E],
    roots: &[String],
    depth: Option<usize>,
) -> Vec<E> {
    if roots.is_empty() {
        return all_edges.to_vec();
    }
    let max_depth = depth.unwrap_or(usize::MAX);
    let mut edges = Vec::new();
    let mut frontier = roots
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    let mut seen_nodes = frontier.clone();
    let mut seen_edges = std::collections::BTreeSet::new();
    for _ in 0..max_depth {
        let mut next = std::collections::BTreeSet::new();
        for edge in all_edges {
            if !frontier.contains(edge.source()) {
                continue;
            }
            if seen_edges.insert(edge.identity()) {
                edges.push(edge.clone());
            }
            if seen_nodes.insert(edge.target().to_string()) {
                next.insert(edge.target().to_string());
            }
        }
        if next.is_empty() {
            break;
        }
        frontier = next;
    }
    edges
}

pub(crate) fn related_edge_view<E>(
    all_edges: &[E],
    roots: &[String],
    direction: EdgeDirection,
) -> Vec<E>
where
    E: IndexedTraversableEdge + Ord,
    E::Kind: std::hash::Hash,
{
    EdgeTraversalIndex::new(all_edges).related(
        roots,
        matches!(direction, EdgeDirection::Dependencies | EdgeDirection::Both),
        matches!(direction, EdgeDirection::Dependents | EdgeDirection::Both),
    )
}
