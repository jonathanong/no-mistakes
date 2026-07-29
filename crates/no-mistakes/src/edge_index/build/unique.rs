use super::super::{CanonicalEdge, EdgeIndex};
#[cfg(debug_assertions)]
use std::collections::HashSet;
use std::hash::Hash;

impl<Node, Kind> EdgeIndex<Node, Kind>
where
    Node: Clone + Eq + Hash + Ord,
    Kind: Clone + Eq + Hash + Ord,
{
    /// Build from canonical edges that are already unique, retaining their
    /// supplied ordinal order without a second deduplication pass.
    ///
    /// Adjacency remains independently sorted below, so callers need only
    /// establish edge uniqueness and their desired canonical edge order.
    pub(crate) fn from_unique_edges_in_order(edges: Vec<CanonicalEdge<Node, Kind>>) -> Self {
        #[cfg(debug_assertions)]
        assert_edges_are_unique(&edges);

        let mut index = Self::default();
        for edge in edges {
            let ordinal = index.edges.len();
            index
                .forward
                .entry(edge.from.clone())
                .or_default()
                .push((edge.to.clone(), edge.kind.clone()));
            index
                .reverse
                .entry(edge.to.clone())
                .or_default()
                .push((edge.from.clone(), edge.kind.clone()));
            index
                .forward_ordinals
                .entry(edge.from.clone())
                .or_default()
                .push(ordinal);
            index
                .reverse_ordinals
                .entry(edge.to.clone())
                .or_default()
                .push(ordinal);
            index.edges.push(edge);
        }
        index.sort_adjacency_by(|left, right| left.cmp(right));
        index
    }
}

#[cfg(debug_assertions)]
fn assert_edges_are_unique<Node, Kind>(edges: &[CanonicalEdge<Node, Kind>])
where
    Node: Eq + Hash,
    Kind: Eq + Hash,
{
    let mut seen = HashSet::with_capacity(edges.len());
    assert!(
        edges.iter().all(|edge| seen.insert(edge)),
        "already-unique edge input must not contain duplicates"
    );
}
