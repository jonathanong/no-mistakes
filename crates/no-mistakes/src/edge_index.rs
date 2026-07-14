use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hash;

mod build;
mod traversal;

#[cfg(test)]
mod tests;

/// One canonical, typed relationship in a request-scoped edge index.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct CanonicalEdge<Node, Kind> {
    pub(crate) from: Node,
    pub(crate) to: Node,
    pub(crate) kind: Kind,
}

impl<Node, Kind> CanonicalEdge<Node, Kind> {
    pub(crate) fn new(from: Node, to: Node, kind: Kind) -> Self {
        Self { from, to, kind }
    }
}

/// Direction used when projecting relationships from an [`EdgeIndex`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum EdgeDirection {
    Dependencies,
    Dependents,
    Both,
}

/// Canonical edges plus deterministic forward and reverse adjacency.
///
/// The edge vector retains the first input ordinal for every unique edge. The
/// adjacency lists are independently sorted and deduplicated for graph walks.
#[derive(Debug, Clone)]
pub(crate) struct EdgeIndex<Node, Kind> {
    edges: Vec<CanonicalEdge<Node, Kind>>,
    forward: HashMap<Node, Vec<(Node, Kind)>>,
    reverse: HashMap<Node, Vec<(Node, Kind)>>,
    forward_ordinals: HashMap<Node, Vec<usize>>,
    reverse_ordinals: HashMap<Node, Vec<usize>>,
}

impl<Node, Kind> Default for EdgeIndex<Node, Kind> {
    fn default() -> Self {
        Self {
            edges: Vec::new(),
            forward: HashMap::new(),
            reverse: HashMap::new(),
            forward_ordinals: HashMap::new(),
            reverse_ordinals: HashMap::new(),
        }
    }
}

impl<Node, Kind> EdgeIndex<Node, Kind>
where
    Node: Clone + Eq + Hash + Ord,
    Kind: Clone + Eq + Hash + Ord,
{
    pub(crate) fn edges(&self) -> &[CanonicalEdge<Node, Kind>] {
        &self.edges
    }

    pub(crate) fn forward(&self) -> &HashMap<Node, Vec<(Node, Kind)>> {
        &self.forward
    }

    pub(crate) fn reverse(&self) -> &HashMap<Node, Vec<(Node, Kind)>> {
        &self.reverse
    }

    pub(crate) fn sort_adjacency_by(
        &mut self,
        mut compare: impl FnMut(&(Node, Kind), &(Node, Kind)) -> Ordering,
    ) {
        for adjacent in self.forward.values_mut().chain(self.reverse.values_mut()) {
            adjacent.sort_by(&mut compare);
            adjacent.dedup();
        }
    }
}
