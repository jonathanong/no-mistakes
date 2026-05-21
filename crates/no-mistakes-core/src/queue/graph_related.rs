use crate::queue::graph::RelatedDirection;
use crate::queue::graph_model::ProjectReport;
use crate::queue::types::Edge;
use crate::queue::utils::{related_from_edges, RelatedEdge};

impl RelatedEdge for Edge {
    fn source(&self) -> &str {
        &self.from
    }

    fn target(&self) -> &str {
        &self.to
    }

    fn reversed(&self) -> Self {
        Edge {
            from: self.to.clone(),
            to: self.from.clone(),
            kind: self.kind,
        }
    }
}

pub fn related(report: &ProjectReport, roots: &[String], direction: RelatedDirection) -> Vec<Edge> {
    related_from_edges(
        &report.edges,
        roots,
        matches!(direction, RelatedDirection::Deps | RelatedDirection::Both),
        matches!(
            direction,
            RelatedDirection::Dependents | RelatedDirection::Both
        ),
    )
}
