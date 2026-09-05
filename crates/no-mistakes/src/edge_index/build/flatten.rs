use super::super::adjacency::{into_adjacency_map, push_ordinal};
use super::super::{CanonicalEdge, EdgeIndex};
use crate::fx::FxHashMap;
use std::cmp::Ordering;
use std::hash::Hash;

impl<Node, Kind> EdgeIndex<Node, Kind>
where
    Node: Clone + Eq + Hash + Ord,
    Kind: Clone + Eq + Hash + Ord,
{
    /// Build from adjacency that has already been sorted and deduplicated.
    ///
    /// Flattening each normalized source adjacency in sorted source order is
    /// equivalent to globally sorting the resulting edges by `(from, to,
    /// kind)`, but avoids materializing and sorting one repository-wide edge
    /// vector. Empty forward nodes and reverse-only targets remain in their
    /// respective maps unchanged.
    #[cfg_attr(not(any(test, feature = "test-instrumentation")), allow(dead_code))]
    pub(crate) fn from_normalized_adjacency_maps_by_source(
        forward: FxHashMap<Node, Vec<(Node, Kind)>>,
        reverse: FxHashMap<Node, Vec<(Node, Kind)>>,
        mut compare_sources: impl FnMut(&Node, &Node) -> Ordering,
    ) -> Self {
        Self::from_normalized_adjacency_maps_with_sorted_sources(forward, reverse, |sources| {
            sources.sort_by(|left, right| compare_sources(left, right))
        })
    }

    /// Same flatten as [`Self::from_normalized_adjacency_maps_by_source`], with
    /// sources ordered by a cached key instead of a comparator.
    pub(crate) fn from_normalized_adjacency_maps_by_cached_source_key<K: Ord>(
        forward: FxHashMap<Node, Vec<(Node, Kind)>>,
        reverse: FxHashMap<Node, Vec<(Node, Kind)>>,
        mut source_key: impl FnMut(&Node) -> K,
    ) -> Self {
        Self::from_normalized_adjacency_maps_with_sorted_sources(forward, reverse, |sources| {
            sources.sort_by_cached_key(|node| source_key(*node))
        })
    }

    fn from_normalized_adjacency_maps_with_sorted_sources(
        forward: FxHashMap<Node, Vec<(Node, Kind)>>,
        reverse: FxHashMap<Node, Vec<(Node, Kind)>>,
        sort_sources: impl FnOnce(&mut Vec<&Node>),
    ) -> Self {
        #[cfg(debug_assertions)]
        super::assert_adjacency_maps_are_consistent(&forward, &reverse);

        let mut forward = into_adjacency_map(forward);
        let mut reverse = into_adjacency_map(reverse);

        let edges =
            crate::perf_trace::trace("graph.canonical_flatten", || {
                let mut sources = forward.keys().collect::<Vec<_>>();
                sort_sources(&mut sources);
                let edge_capacity = forward.values().map(|adj| adj.neighbors.len()).sum();
                let mut edges = Vec::with_capacity(edge_capacity);
                for from in sources {
                    edges.extend(forward[from].neighbors.iter().map(|(to, kind)| {
                        CanonicalEdge::new(from.clone(), to.clone(), kind.clone())
                    }));
                }
                edges
            });

        crate::perf_trace::trace("graph.ordinal_construction", || {
            for (ordinal, edge) in edges.iter().enumerate() {
                push_ordinal(&mut forward, &edge.from, ordinal);
                push_ordinal(&mut reverse, &edge.to, ordinal);
            }
        });

        Self {
            edges,
            forward,
            reverse,
        }
    }
}
