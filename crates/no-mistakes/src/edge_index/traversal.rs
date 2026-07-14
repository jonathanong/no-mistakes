use super::{CanonicalEdge, EdgeDirection, EdgeIndex, NodeAliases};
use std::collections::{BTreeSet, HashSet};
use std::hash::Hash;

impl<Node, Kind> EdgeIndex<Node, Kind>
where
    Node: Clone + Eq + Hash + Ord,
    Kind: Clone + Eq + Hash + Ord,
{
    /// Traverse edges in level order, preserving their canonical input ordinal
    /// within each level. Reverse walks return edges in traversal orientation.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn traverse(
        &self,
        roots: &[Node],
        direction: EdgeDirection,
        max_depth: Option<usize>,
    ) -> Vec<CanonicalEdge<Node, Kind>> {
        self.traverse_impl(roots, direction, max_depth, None)
    }

    /// Traverse while treating each mapped node as equivalent to its aliases.
    ///
    /// Aliases affect reachability only: returned edges always come from the
    /// canonical index and never include synthetic equivalence edges.
    pub(crate) fn traverse_with_aliases(
        &self,
        roots: &[Node],
        direction: EdgeDirection,
        max_depth: Option<usize>,
        aliases: &NodeAliases<Node>,
    ) -> Vec<CanonicalEdge<Node, Kind>> {
        self.traverse_impl(roots, direction, max_depth, Some(aliases))
    }

    fn traverse_impl(
        &self,
        roots: &[Node],
        direction: EdgeDirection,
        max_depth: Option<usize>,
        aliases: Option<&NodeAliases<Node>>,
    ) -> Vec<CanonicalEdge<Node, Kind>> {
        let mut frontier = expand_aliases(roots.iter().cloned(), aliases);
        let mut seen_nodes = frontier.iter().cloned().collect::<HashSet<_>>();
        let mut seen_arcs = HashSet::new();
        let mut emitted_edges = HashSet::new();
        let mut output = Vec::new();
        let max_depth = max_depth.unwrap_or(usize::MAX);

        for _ in 0..max_depth {
            let mut arcs = BTreeSet::new();
            for node in &frontier {
                if matches!(direction, EdgeDirection::Dependencies | EdgeDirection::Both) {
                    if let Some(ordinals) = self.forward_ordinals.get(node) {
                        arcs.extend(ordinals.iter().map(|ordinal| (*ordinal, false)));
                    }
                }
                if matches!(direction, EdgeDirection::Dependents | EdgeDirection::Both) {
                    if let Some(ordinals) = self.reverse_ordinals.get(node) {
                        arcs.extend(ordinals.iter().map(|ordinal| (*ordinal, true)));
                    }
                }
            }

            let mut next = BTreeSet::new();
            for (ordinal, reversed) in arcs {
                if !seen_arcs.insert((ordinal, reversed)) {
                    continue;
                }
                let edge = &self.edges[ordinal];
                let (from, to) = if reversed {
                    (&edge.to, &edge.from)
                } else {
                    (&edge.from, &edge.to)
                };
                let projected = CanonicalEdge::new(from.clone(), to.clone(), edge.kind.clone());
                if emitted_edges.insert(projected.clone()) {
                    output.push(projected);
                }
                if seen_nodes.insert(to.clone()) {
                    for node in expand_aliases(std::iter::once(to.clone()), aliases) {
                        if seen_nodes.insert(node.clone()) || node == *to {
                            next.insert(node);
                        }
                    }
                }
            }
            if next.is_empty() {
                break;
            }
            frontier = next;
        }
        output
    }
}

fn expand_aliases<Node>(
    nodes: impl IntoIterator<Item = Node>,
    aliases: Option<&NodeAliases<Node>>,
) -> BTreeSet<Node>
where
    Node: Clone + Eq + Hash + Ord,
{
    match aliases {
        Some(aliases) => aliases.expand(nodes),
        None => nodes.into_iter().collect(),
    }
}
