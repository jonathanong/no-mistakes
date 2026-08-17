use super::adjacency::{into_adjacency_map, push_ordinal};
use super::{CanonicalEdge, EdgeIndex};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

impl<Node, Kind> EdgeIndex<Node, Kind>
where
    Node: Clone + Eq + Hash + Ord,
    Kind: Clone + Eq + Hash + Ord,
{
    /// Test-only legacy constructor used by the global-sort oracle.
    pub(crate) fn from_adjacency_maps_by(
        forward: HashMap<Node, Vec<(Node, Kind)>>,
        reverse: HashMap<Node, Vec<(Node, Kind)>>,
        mut compare: impl FnMut(&CanonicalEdge<Node, Kind>, &CanonicalEdge<Node, Kind>) -> Ordering,
    ) -> Self {
        assert_adjacency_maps_are_consistent(&forward, &reverse);

        let mut edges = Vec::with_capacity(forward.values().map(Vec::len).sum());
        for (from, adjacent) in &forward {
            for (to, kind) in adjacent {
                edges.push(CanonicalEdge::new(from.clone(), to.clone(), kind.clone()));
            }
        }
        edges.sort_by(&mut compare);
        edges.dedup();

        let mut forward = into_adjacency_map(forward);
        let mut reverse = into_adjacency_map(reverse);
        for (ordinal, edge) in edges.iter().enumerate() {
            push_ordinal(&mut forward, &edge.from, ordinal);
            push_ordinal(&mut reverse, &edge.to, ordinal);
        }

        Self {
            edges,
            forward,
            reverse,
        }
    }
}

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
