use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hash;

mod adjacency;
mod aliases;
mod build;
mod prepared;
mod traversal;

pub(crate) use adjacency::Adjacency;
pub(crate) use aliases::NodeAliases;
pub(crate) use prepared::PreparedRelationshipIndex;

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_extend;

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
    forward: HashMap<Node, Adjacency<Node, Kind>>,
    reverse: HashMap<Node, Adjacency<Node, Kind>>,
}

impl<Node, Kind> Default for EdgeIndex<Node, Kind> {
    fn default() -> Self {
        Self {
            edges: Vec::new(),
            forward: HashMap::new(),
            reverse: HashMap::new(),
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

    pub(crate) fn forward(&self) -> &HashMap<Node, Adjacency<Node, Kind>> {
        &self.forward
    }

    pub(crate) fn reverse(&self) -> &HashMap<Node, Adjacency<Node, Kind>> {
        &self.reverse
    }

    pub(crate) fn sort_adjacency_by(
        &mut self,
        mut compare: impl FnMut(&(Node, Kind), &(Node, Kind)) -> Ordering,
    ) {
        for adjacent in self.forward.values_mut().chain(self.reverse.values_mut()) {
            adjacent.sort_by(&mut compare);
        }
    }

    pub(crate) fn sort_adjacency_by_cached_key<K>(
        &mut self,
        mut key: impl FnMut(&(Node, Kind)) -> K,
    ) where
        K: Ord,
    {
        for adjacent in self.forward.values_mut().chain(self.reverse.values_mut()) {
            adjacent.sort_by_cached_key(&mut key);
        }
    }
}
