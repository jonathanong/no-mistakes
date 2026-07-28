use super::{CanonicalEdge, EdgeDirection, EdgeIndex, NodeAliases};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::hash::Hash;

/// Typed relationships prepared once for public-name root lookup and projection.
///
/// Domains keep ownership of their public report and rendering types while this
/// helper centralizes the shared typed traversal, alias expansion, and stable
/// first-seen projection behavior.
#[derive(Debug, Clone)]
pub(crate) struct PreparedRelationshipIndex<Node, Kind> {
    index: EdgeIndex<Node, Kind>,
    nodes_by_name: HashMap<String, Vec<Node>>,
    aliases: NodeAliases<Node>,
}

impl<Node, Kind> PreparedRelationshipIndex<Node, Kind>
where
    Node: Clone + Eq + Hash + Ord,
    Kind: Clone + Eq + Hash + Ord,
{
    /// Normalize typed relationships in their public display order and retain
    /// every typed node that shares a public root identity as an alias group.
    pub(crate) fn from_edges(
        edges: impl IntoIterator<Item = CanonicalEdge<Node, Kind>>,
        mut public_node: impl FnMut(&Node) -> String,
    ) -> Self {
        let mut edges = edges.into_iter().collect::<Vec<_>>();
        edges.sort_by(|left, right| {
            public_node(&left.from)
                .cmp(&public_node(&right.from))
                .then_with(|| public_node(&left.to).cmp(&public_node(&right.to)))
                .then_with(|| left.kind.cmp(&right.kind))
                .then_with(|| left.cmp(right))
        });
        edges.dedup();

        // A BTreeSet preserves the pre-existing sorted typed-root vectors
        // without repeatedly scanning a high-collision public-name bucket.
        let mut nodes_by_name = HashMap::<String, BTreeSet<Node>>::new();
        for edge in &edges {
            for node in [&edge.from, &edge.to] {
                nodes_by_name
                    .entry(public_node(node))
                    .or_default()
                    .insert(node.clone());
            }
        }
        let aliases = NodeAliases::from_groups(
            nodes_by_name
                .values()
                .map(|nodes| nodes.iter().cloned().collect::<Vec<_>>()),
        );
        let nodes_by_name = nodes_by_name
            .into_iter()
            .map(|(name, nodes)| (name, nodes.into_iter().collect()))
            .collect();

        Self {
            index: EdgeIndex::from_edges(edges),
            nodes_by_name,
            aliases,
        }
    }

    /// Project the dependencies view used by the `edges` command. With no
    /// roots, this intentionally returns every canonical edge in display
    /// order; related queries retain their historical empty-root result.
    pub(crate) fn edge_view<Output>(
        &self,
        roots: &[String],
        depth: Option<usize>,
        project: impl FnMut(CanonicalEdge<Node, Kind>) -> Output,
    ) -> Vec<Output>
    where
        Output: Clone + Eq + Hash,
    {
        let relationships = if roots.is_empty() {
            self.index.edges().to_vec()
        } else {
            self.traverse(roots, EdgeDirection::Dependencies, depth)
        };
        project_first_seen(relationships, project)
    }

    /// Project a related view. Reverse edges retain traversal orientation; the
    /// caller is responsible for any domain-specific final sorting.
    pub(crate) fn related<Output>(
        &self,
        roots: &[String],
        direction: EdgeDirection,
        project: impl FnMut(CanonicalEdge<Node, Kind>) -> Output,
    ) -> Vec<Output>
    where
        Output: Clone + Eq + Hash,
    {
        project_first_seen(self.traverse(roots, direction, None), project)
    }

    fn traverse(
        &self,
        roots: &[String],
        direction: EdgeDirection,
        depth: Option<usize>,
    ) -> Vec<CanonicalEdge<Node, Kind>> {
        self.index
            .traverse_with_aliases(&self.typed_roots(roots), direction, depth, &self.aliases)
    }

    fn typed_roots(&self, roots: &[String]) -> Vec<Node> {
        roots
            .iter()
            .flat_map(|root| self.nodes_by_name.get(root).into_iter().flatten().cloned())
            .collect()
    }
}

fn project_first_seen<Input, Output>(
    values: impl IntoIterator<Item = Input>,
    mut project: impl FnMut(Input) -> Output,
) -> Vec<Output>
where
    Output: Clone + Eq + Hash,
{
    let mut seen = HashSet::new();
    values
        .into_iter()
        .filter_map(|value| {
            let projected = project(value);
            seen.insert(projected.clone()).then_some(projected)
        })
        .collect()
}

#[cfg(test)]
mod test_support;
