use super::super::adjacency::{push_neighbor, seed_known_targets};
use super::super::{CanonicalEdge, EdgeIndex};
use crate::fx::{fx_map_with_capacity, FxHashMap, FxHashSet};
use std::hash::Hash;

impl<Node, Kind> EdgeIndex<Node, Kind>
where
    Node: Clone + Eq + Hash + Ord,
    Kind: Clone + Eq + Hash + Ord,
{
    /// Append edges without renumbering existing canonical ordinals.
    ///
    /// Callers that need a domain-specific adjacency order can sort it after
    /// this operation; ordinals intentionally retain base edges before new
    /// edges, matching reconstruction through [`Self::from_edges_and_nodes`].
    pub(crate) fn extend_edges_preserving_ordinals(
        &mut self,
        edges: impl IntoIterator<Item = CanonicalEdge<Node, Kind>>,
    ) {
        let edges = edges.into_iter();
        let (lower, upper) = edges.size_hint();
        let mut known_by_source = fx_map_with_capacity(upper.unwrap_or(lower));
        for edge in edges {
            if !known_by_source.contains_key(&edge.from) {
                known_by_source.insert(
                    edge.from.clone(),
                    seed_known_targets(self.forward.get(&edge.from)),
                );
            }
            if pair_is_known(&known_by_source[&edge.from], &edge.to, &edge.kind) {
                continue;
            }
            let known = known_by_source
                .get_mut(&edge.from)
                .expect("source known-set is inserted before accept");
            known
                .entry(edge.to.clone())
                .or_default()
                .insert(edge.kind.clone());
            let ordinal = self.edges.len();
            push_neighbor(
                &mut self.forward,
                &edge.from,
                (edge.to.clone(), edge.kind.clone()),
                ordinal,
            );
            push_neighbor(
                &mut self.reverse,
                &edge.to,
                (edge.from.clone(), edge.kind.clone()),
                ordinal,
            );
            self.edges.push(edge);
        }
    }
}

fn pair_is_known<Node, Kind>(
    known: &FxHashMap<Node, FxHashSet<Kind>>,
    to: &Node,
    kind: &Kind,
) -> bool
where
    Node: Eq + Hash,
    Kind: Eq + Hash,
{
    known.get(to).is_some_and(|kinds| kinds.contains(kind))
}
