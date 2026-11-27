use super::{Edge, EdgeKind};
use crate::cli::traversal::{IndexedTraversableEdge, TraversableEdge};

impl TraversableEdge for Edge {
    type Kind = EdgeKind;

    fn source(&self) -> &str {
        &self.from
    }

    fn target(&self) -> &str {
        &self.to
    }

    fn kind(&self) -> Self::Kind {
        self.kind
    }
}

impl IndexedTraversableEdge for Edge {
    fn reversed(&self) -> Self {
        Self {
            from: self.to.clone(),
            to: self.from.clone(),
            kind: self.kind,
        }
    }
}
