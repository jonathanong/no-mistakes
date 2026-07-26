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

    /// Build from adjacency that has already been sorted and deduplicated.
    ///
    /// Flattening each normalized source adjacency in sorted source order is
    /// equivalent to globally sorting the resulting edges by `(from, to,
    /// kind)`, but avoids materializing and sorting one repository-wide edge
    /// vector. Empty forward nodes and reverse-only targets remain in their
    /// respective maps unchanged.
    pub(crate) fn from_normalized_adjacency_maps_by_source(
        forward: HashMap<Node, Vec<(Node, Kind)>>,
        reverse: HashMap<Node, Vec<(Node, Kind)>>,
        mut compare_sources: impl FnMut(&Node, &Node) -> Ordering,
    ) -> Self {
        #[cfg(debug_assertions)]
        assert_adjacency_maps_are_consistent(&forward, &reverse);

        let edges =
            crate::perf_trace::trace("graph.canonical_flatten", || {
                let mut sources = forward.keys().collect::<Vec<_>>();
                sources.sort_by(|left, right| compare_sources(left, right));
                let edge_capacity = forward.values().map(Vec::len).sum();
                let mut edges = Vec::with_capacity(edge_capacity);
                for from in sources {
                    edges.extend(forward[from].iter().map(|(to, kind)| {
                        CanonicalEdge::new(from.clone(), to.clone(), kind.clone())
                    }));
                }
                edges
            });

        let (forward_ordinals, reverse_ordinals) =
            crate::perf_trace::trace("graph.ordinal_construction", || {
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
                (forward_ordinals, reverse_ordinals)
            });

        Self {
            edges,
            forward,
            reverse,
            forward_ordinals,
            reverse_ordinals,
        }
    }

    /// Append edges without renumbering existing canonical ordinals.
    ///
    /// Callers that need a domain-specific adjacency order can sort it after
    /// this operation; ordinals intentionally retain base edges before new
    /// edges, matching reconstruction through [`Self::from_edges_and_nodes`].
    pub(crate) fn extend_edges_preserving_ordinals(
        &mut self,
        edges: impl IntoIterator<Item = CanonicalEdge<Node, Kind>>,
    ) {
        // Materialize each touched source's existing adjacency once. This
        // keeps high-fanout selector batches linear in that source's existing
        // and incoming edges rather than scanning a growing Vec per edge.
        let mut known_by_source = HashMap::<Node, HashSet<(Node, Kind)>>::new();
        for edge in edges {
            let known = known_by_source.entry(edge.from.clone()).or_insert_with(|| {
                self.forward
                    .get(&edge.from)
                    .map(|adjacent| adjacent.iter().cloned().collect())
                    .unwrap_or_default()
            });
            if !known.insert((edge.to.clone(), edge.kind.clone())) {
                continue;
            }
            let ordinal = self.edges.len();
            self.forward
                .entry(edge.from.clone())
                .or_default()
                .push((edge.to.clone(), edge.kind.clone()));
            self.reverse
                .entry(edge.to.clone())
                .or_default()
                .push((edge.from.clone(), edge.kind.clone()));
            self.forward_ordinals
                .entry(edge.from.clone())
                .or_default()
                .push(ordinal);
            self.reverse_ordinals
                .entry(edge.to.clone())
                .or_default()
                .push(ordinal);
            self.edges.push(edge);
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
