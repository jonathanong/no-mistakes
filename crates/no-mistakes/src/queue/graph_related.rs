use crate::cli::related_edge_view;
use crate::edge_index::EdgeDirection;
use crate::queue::graph::RelatedDirection;
use crate::queue::graph_build::public_node;
use crate::queue::graph_model::{PreparedProjectReport, ProjectReport};
use crate::queue::types::Edge;

pub fn related(report: &ProjectReport, roots: &[String], direction: RelatedDirection) -> Vec<Edge> {
    related_edge_view(
        &report.edges,
        roots,
        match direction {
            RelatedDirection::Deps => EdgeDirection::Dependencies,
            RelatedDirection::Dependents => EdgeDirection::Dependents,
            RelatedDirection::Both => EdgeDirection::Both,
        },
    )
}

impl PreparedProjectReport {
    pub fn edge_view(&self, roots: &[String], depth: Option<usize>) -> Vec<Edge> {
        if roots.is_empty() {
            return self.report.edges.clone();
        }
        self.project(self.index.traverse_with_aliases(
            &self.typed_roots(roots),
            EdgeDirection::Dependencies,
            depth,
            &self.aliases,
        ))
    }

    pub fn related(&self, roots: &[String], direction: RelatedDirection) -> Vec<Edge> {
        let direction = match direction {
            RelatedDirection::Deps => EdgeDirection::Dependencies,
            RelatedDirection::Dependents => EdgeDirection::Dependents,
            RelatedDirection::Both => EdgeDirection::Both,
        };
        let mut edges = self.project(self.index.traverse_with_aliases(
            &self.typed_roots(roots),
            direction,
            None,
            &self.aliases,
        ));
        edges.sort();
        edges.dedup();
        edges
    }

    fn typed_roots(&self, roots: &[String]) -> Vec<crate::queue::types::RelationshipNode> {
        roots
            .iter()
            .flat_map(|root| self.nodes_by_name.get(root).into_iter().flatten().cloned())
            .collect()
    }

    fn project(
        &self,
        relationships: Vec<
            crate::edge_index::CanonicalEdge<
                crate::queue::types::RelationshipNode,
                crate::queue::EdgeKind,
            >,
        >,
    ) -> Vec<Edge> {
        let mut edges = Vec::new();
        for relationship in relationships {
            let edge = Edge {
                from: public_node(&self.root, &relationship.from),
                to: public_node(&self.root, &relationship.to),
                kind: relationship.kind,
            };
            if !edges.contains(&edge) {
                edges.push(edge);
            }
        }
        edges
    }
}
