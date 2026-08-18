use super::adjacency;
use super::{CanonicalEdge, EdgeIndex};
#[cfg(debug_assertions)]
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;

mod extend;
mod flatten;
mod unique;

impl<Node, Kind> EdgeIndex<Node, Kind>
where
    Node: Clone + Eq + Hash + Ord,
    Kind: Clone + Eq + Hash + Ord,
{
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn from_edges(edges: impl IntoIterator<Item = CanonicalEdge<Node, Kind>>) -> Self {
        Self::from_edges_and_nodes(edges, std::iter::empty())
    }

    pub(crate) fn from_edges_and_nodes(
        edges: impl IntoIterator<Item = CanonicalEdge<Node, Kind>>,
        nodes: impl IntoIterator<Item = Node>,
    ) -> Self {
        let mut index = Self::default();
        for node in nodes {
            index.forward.entry(node).or_default();
        }

        let mut seen = HashSet::new();
        for edge in edges {
            if !seen.insert(edge.clone()) {
                continue;
            }
            let ordinal = index.edges.len();
            adjacency::push_neighbor(
                &mut index.forward,
                &edge.from,
                (edge.to.clone(), edge.kind.clone()),
                ordinal,
            );
            adjacency::push_neighbor(
                &mut index.reverse,
                &edge.to,
                (edge.from.clone(), edge.kind.clone()),
                ordinal,
            );
            index.edges.push(edge);
        }
        index.sort_adjacency_by(|left, right| left.cmp(right));
        index
    }
}

#[cfg(debug_assertions)]
pub(super) fn assert_adjacency_maps_are_consistent<Node, Kind>(
    forward: &HashMap<Node, Vec<(Node, Kind)>>,
    reverse: &HashMap<Node, Vec<(Node, Kind)>>,
) where
    Node: Clone + Eq + Hash,
    Kind: Clone + Eq + Hash,
{
    let forward_edges = forward
        .iter()
        .flat_map(|(from, adjacent)| {
            adjacent
                .iter()
                .map(|(to, kind)| (from.clone(), to.clone(), kind.clone()))
        })
        .collect::<HashSet<_>>();
    let reverse_edges = reverse
        .iter()
        .flat_map(|(to, adjacent)| {
            adjacent
                .iter()
                .map(|(from, kind)| (from.clone(), to.clone(), kind.clone()))
        })
        .collect::<HashSet<_>>();
    assert!(
        forward_edges == reverse_edges,
        "forward and reverse adjacency maps must describe identical edges"
    );
}
