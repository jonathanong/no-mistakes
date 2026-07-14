use super::{CanonicalEdge, EdgeIndex};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

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

    pub(crate) fn from_adjacency_maps_by(
        forward: HashMap<Node, Vec<(Node, Kind)>>,
        reverse: HashMap<Node, Vec<(Node, Kind)>>,
        mut compare: impl FnMut(&CanonicalEdge<Node, Kind>, &CanonicalEdge<Node, Kind>) -> Ordering,
    ) -> Self {
        #[cfg(debug_assertions)]
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

#[cfg(debug_assertions)]
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
