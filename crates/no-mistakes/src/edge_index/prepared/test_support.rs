use super::*;

impl<Node, Kind> PreparedRelationshipIndex<Node, Kind>
where
    Node: Clone + Eq + Hash + Ord,
    Kind: Clone + Eq + Hash + Ord,
{
    pub(crate) fn edges(&self) -> &[CanonicalEdge<Node, Kind>] {
        self.index.edges()
    }
}
