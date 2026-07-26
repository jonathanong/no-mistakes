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
            edges.extend(
                adjacent
                    .iter()
                    .map(|(to, kind)| CanonicalEdge::new(from.clone(), to.clone(), kind.clone())),
            );
        }
        edges.sort_by(&mut compare);
        edges.dedup();

        let mut forward_ordinals: HashMap<Node, Vec<usize>> = HashMap::new();
        let mut reverse_ordinals: HashMap<Node, Vec<usize>> = HashMap::new();
        for (ordinal, edge) in edges.iter().enumerate() {
            forward_ordinals
                .entry(edge.from.clone())
                .or_default()
                .push(ordinal);
            reverse_ordinals
                .entry(edge.to.clone())
                .or_default()
                .push(ordinal);
        }

        Self {
            edges,
            forward,
            reverse,
            forward_ordinals,
            reverse_ordinals,
        }
    }
}

fn assert_adjacency_maps_are_consistent<Node, Kind>(
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
