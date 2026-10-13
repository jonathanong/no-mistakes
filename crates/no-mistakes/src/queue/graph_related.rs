use crate::cli::related_edge_view;
use crate::queue::graph::RelatedDirection;
use crate::queue::graph_model::{PreparedProjectReport, ProjectReport};
use crate::queue::types::Edge;

impl From<RelatedDirection> for crate::edge_index::EdgeDirection {
    fn from(direction: RelatedDirection) -> Self {
        match direction {
            RelatedDirection::Deps => Self::Dependencies,
            RelatedDirection::Dependents => Self::Dependents,
            RelatedDirection::Both => Self::Both,
        }
    }
}

impl From<(String, String, crate::queue::types::EdgeKind)> for Edge {
    fn from((from, to, kind): (String, String, crate::queue::types::EdgeKind)) -> Self {
        Self { from, to, kind }
    }
}

pub fn related(report: &ProjectReport, roots: &[String], direction: RelatedDirection) -> Vec<Edge> {
    related_edge_view(&report.edges, roots, direction.into())
}

impl PreparedProjectReport {
    pub fn edge_view(&self, roots: &[String], depth: Option<usize>) -> Vec<Edge> {
        crate::cli::prepared_edge_view(
            &self.relationships,
            roots,
            depth,
            crate::cli::project_relationship_edge,
        )
    }

    pub fn related(&self, roots: &[String], direction: RelatedDirection) -> Vec<Edge> {
        crate::cli::prepared_related_view(
            &self.relationships,
            roots,
            direction.into(),
            crate::cli::project_relationship_edge,
        )
    }
}
