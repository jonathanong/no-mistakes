use crate::fx::{fx_map_with_capacity, FxHashMap, FxHashSet};
use std::hash::Hash;
use std::ops::Deref;

/// Forward or reverse adjacency for one node.
///
/// Neighbors stay independently sortable for public walks. Ordinals stay in
/// first-seen order so traversal does not follow a later adjacency sort.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Adjacency<Node, Kind> {
    pub(crate) neighbors: Vec<(Node, Kind)>,
    pub(crate) ordinals: Vec<usize>,
}

impl<Node, Kind> Default for Adjacency<Node, Kind> {
    fn default() -> Self {
        Self {
            neighbors: Vec::new(),
            ordinals: Vec::new(),
        }
    }
}

impl<Node, Kind> Adjacency<Node, Kind> {
    pub(crate) fn from_neighbors(neighbors: Vec<(Node, Kind)>) -> Self {
        let ordinals = Vec::with_capacity(neighbors.len());
        Self {
            neighbors,
            ordinals,
        }
    }

    pub(crate) fn from_one(neighbor: (Node, Kind), ordinal: usize) -> Self {
        Self {
            neighbors: vec![neighbor],
            ordinals: vec![ordinal],
        }
    }

    pub(crate) fn push(&mut self, neighbor: (Node, Kind), ordinal: usize) {
        self.neighbors.push(neighbor);
        self.ordinals.push(ordinal);
    }

    pub(crate) fn sort_by(
        &mut self,
        compare: impl FnMut(&(Node, Kind), &(Node, Kind)) -> std::cmp::Ordering,
    ) where
        Node: PartialEq,
        Kind: PartialEq,
    {
        self.neighbors.sort_by(compare);
        self.neighbors.dedup();
    }

    pub(crate) fn sort_by_cached_key<K: Ord>(&mut self, key: impl FnMut(&(Node, Kind)) -> K)
    where
        Node: PartialEq,
        Kind: PartialEq,
    {
        self.neighbors.sort_by_cached_key(key);
        self.neighbors.dedup();
    }
}

impl<Node, Kind> AsRef<[(Node, Kind)]> for Adjacency<Node, Kind> {
    fn as_ref(&self) -> &[(Node, Kind)] {
        &self.neighbors
    }
}

impl<Node, Kind> Deref for Adjacency<Node, Kind> {
    type Target = [(Node, Kind)];

    fn deref(&self) -> &Self::Target {
        &self.neighbors
    }
}

impl<Node: PartialEq, Kind: PartialEq> PartialEq<[(Node, Kind)]> for Adjacency<Node, Kind> {
    fn eq(&self, other: &[(Node, Kind)]) -> bool {
        self.neighbors == other
    }
}

impl<Node: PartialEq, Kind: PartialEq> PartialEq<Vec<(Node, Kind)>> for Adjacency<Node, Kind> {
    fn eq(&self, other: &Vec<(Node, Kind)>) -> bool {
        &self.neighbors == other
    }
}

impl<'a, Node, Kind> IntoIterator for &'a Adjacency<Node, Kind> {
    type Item = &'a (Node, Kind);
    type IntoIter = std::slice::Iter<'a, (Node, Kind)>;

    fn into_iter(self) -> Self::IntoIter {
        self.neighbors.iter()
    }
}

pub(crate) fn into_adjacency_map<Node, Kind>(
    map: FxHashMap<Node, Vec<(Node, Kind)>>,
) -> FxHashMap<Node, Adjacency<Node, Kind>>
where
    Node: Eq + Hash,
{
    let mut out = fx_map_with_capacity(map.len());
    for (node, neighbors) in map {
        out.insert(node, Adjacency::from_neighbors(neighbors));
    }
    out
}

pub(crate) fn push_neighbor<Node, Kind>(
    map: &mut FxHashMap<Node, Adjacency<Node, Kind>>,
    key: &Node,
    neighbor: (Node, Kind),
    ordinal: usize,
) where
    Node: Clone + Eq + Hash,
{
    match map.get_mut(key) {
        Some(adj) => adj.push(neighbor, ordinal),
        None => {
            map.insert(key.clone(), Adjacency::from_one(neighbor, ordinal));
        }
    }
}

pub(crate) fn push_ordinal<Node, Kind>(
    map: &mut FxHashMap<Node, Adjacency<Node, Kind>>,
    key: &Node,
    ordinal: usize,
) where
    Node: Clone + Eq + Hash,
{
    match map.get_mut(key) {
        Some(adj) => adj.ordinals.push(ordinal),
        None => {
            let mut adj = Adjacency::default();
            adj.ordinals.push(ordinal);
            map.insert(key.clone(), adj);
        }
    }
}

pub(crate) fn remember_pair<Node, Kind>(
    known: &mut FxHashMap<Node, FxHashSet<Kind>>,
    to: &Node,
    kind: &Kind,
) -> bool
where
    Node: Clone + Eq + Hash,
    Kind: Clone + Eq + Hash,
{
    if let Some(kinds) = known.get_mut(to) {
        return kinds.insert(kind.clone());
    }
    let mut kinds = FxHashSet::default();
    kinds.insert(kind.clone());
    known.insert(to.clone(), kinds);
    true
}

pub(crate) fn seed_known_targets<Node, Kind>(
    existing: Option<&Adjacency<Node, Kind>>,
) -> FxHashMap<Node, FxHashSet<Kind>>
where
    Node: Clone + Eq + Hash,
    Kind: Clone + Eq + Hash,
{
    let Some(adj) = existing else {
        return FxHashMap::default();
    };
    if adj.neighbors.is_empty() {
        return FxHashMap::default();
    }
    let mut known: FxHashMap<Node, FxHashSet<Kind>> = fx_map_with_capacity(adj.neighbors.len());
    for (to, kind) in &adj.neighbors {
        remember_pair(&mut known, to, kind);
    }
    known
}

#[cfg(test)]
mod tests;
