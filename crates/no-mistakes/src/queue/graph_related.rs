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
        let root = &self.root;
        self.relationships
            .edge_view(roots, depth, |relationship| Edge {
                from: public_node(root, &relationship.from),
                to: public_node(root, &relationship.to),
                kind: relationship.kind,
            })
    }

    pub fn related(&self, roots: &[String], direction: RelatedDirection) -> Vec<Edge> {
        let direction = match direction {
            RelatedDirection::Deps => EdgeDirection::Dependencies,
            RelatedDirection::Dependents => EdgeDirection::Dependents,
            RelatedDirection::Both => EdgeDirection::Both,
        };
        let root = &self.root;
        let mut edges = self
            .relationships
            .related(roots, direction, |relationship| Edge {
                from: public_node(root, &relationship.from),
                to: public_node(root, &relationship.to),
                kind: relationship.kind,
            });
        edges.sort();
        edges.dedup();
        edges
    }
}
