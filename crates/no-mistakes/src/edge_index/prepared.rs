use super::{CanonicalEdge, EdgeDirection, EdgeIndex, NodeAliases};
use crate::fx::{fx_map_with_capacity, FxHashMap};
use std::collections::HashSet;
use std::hash::Hash;

/// Typed relationships prepared once for public-name root lookup and projection.
///
/// Domains keep ownership of their public report and rendering types while this
/// helper centralizes the shared typed traversal, alias expansion, and stable
/// first-seen projection behavior.
#[derive(Debug, Clone)]
pub(crate) struct PreparedRelationshipIndex<Node, Kind> {
    index: EdgeIndex<Node, Kind>,
    public_names: FxHashMap<Node, String>,
    nodes_by_name: FxHashMap<String, Vec<Node>>,
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
        let edges = edges.into_iter().collect::<Vec<_>>();
        // A typed node often appears in many relationships. Public rendering
        // can require path normalization/allocation, so derive it once per
        // distinct typed node before sorting or grouping its edges.
        let mut public_names = fx_map_with_capacity(edges.len());
        for edge in &edges {
            public_names
                .entry(edge.from.clone())
                .or_insert_with(|| public_node(&edge.from));
            public_names
                .entry(edge.to.clone())
                .or_insert_with(|| public_node(&edge.to));
        }
        // First-pass capacity is raw edge count; unique nodes are fewer when
        // the same job is enqueued from many sites. Drop the spare buckets so
        // the retained index does not keep that construction over-allocation.
        public_names.shrink_to_fit();
        let public_name = |node: &Node| {
            public_names
                .get(node)
                .expect("every indexed relationship node must have a cached public name")
        };
        let mut edges = edges;
        edges.sort_by(|left, right| {
            public_name(&left.from)
                .cmp(public_name(&right.from))
                .then_with(|| public_name(&left.to).cmp(public_name(&right.to)))
                .then_with(|| left.kind.cmp(&right.kind))
                .then_with(|| left.cmp(right))
        });
        edges.dedup();

        // Sort/dedup each bucket once so typed-root vectors stay ordered.
        let mut grouped_nodes: FxHashMap<&str, Vec<Node>> = fx_map_with_capacity(edges.len());
        for edge in &edges {
            grouped_nodes
                .entry(public_name(&edge.from).as_str())
                .or_default()
                .push(edge.from.clone());
            grouped_nodes
                .entry(public_name(&edge.to).as_str())
                .or_default()
                .push(edge.to.clone());
        }
        for nodes in grouped_nodes.values_mut() {
            nodes.sort();
            nodes.dedup();
        }
        let aliases = NodeAliases::from_groups(grouped_nodes.values().cloned());
        let mut nodes_by_name = fx_map_with_capacity(grouped_nodes.len());
        for (name, nodes) in grouped_nodes {
            nodes_by_name.insert(name.to_owned(), nodes);
        }

        Self {
            index: EdgeIndex::from_unique_edges_in_order(edges),
            public_names,
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
        mut project: impl FnMut(&CanonicalEdge<Node, Kind>, &str, &str) -> Output,
    ) -> Vec<Output>
    where
        Output: Clone + Eq + Hash,
    {
        if roots.is_empty() {
            project_first_seen(self.index.edges().iter(), |edge| {
                project(
                    edge,
                    self.public_name(&edge.from),
                    self.public_name(&edge.to),
                )
            })
        } else {
            let relationships = self.traverse(roots, EdgeDirection::Dependencies, depth);
            project_first_seen(relationships.iter(), |edge| {
                project(
                    edge,
                    self.public_name(&edge.from),
                    self.public_name(&edge.to),
                )
            })
        }
    }

    /// Project a related view. Reverse edges retain traversal orientation; the
    /// caller is responsible for any domain-specific final sorting.
    pub(crate) fn related<Output>(
        &self,
        roots: &[String],
        direction: EdgeDirection,
        mut project: impl FnMut(&CanonicalEdge<Node, Kind>, &str, &str) -> Output,
    ) -> Vec<Output>
    where
        Output: Clone + Eq + Hash,
    {
        let relationships = self.traverse(roots, direction, None);
        project_first_seen(relationships.iter(), |edge| {
            project(
                edge,
                self.public_name(&edge.from),
                self.public_name(&edge.to),
            )
        })
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

    fn public_name(&self, node: &Node) -> &str {
        self.public_names
            .get(node)
            .map(String::as_str)
            .expect("every indexed relationship node must have a cached public name")
    }
}

fn project_first_seen<Input, Output>(
    values: impl IntoIterator<Item = Input>,
    mut project: impl FnMut(Input) -> Output,
) -> Vec<Output>
where
    Output: Clone + Eq + Hash,
{
    // Projected `Output` is a public report edge (paths/jobs from repo
    // source), not an interned analysis key. Keep SipHash here.
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
