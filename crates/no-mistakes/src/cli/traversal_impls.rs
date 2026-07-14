use crate::cli::traversal::{IndexedTraversableEdge, TraversableEdge};

impl TraversableEdge for crate::queue::Edge {
    type Kind = crate::queue::EdgeKind;

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

impl TraversableEdge for crate::server_routes::Edge {
    type Kind = crate::server_routes::EdgeKind;

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
impl IndexedTraversableEdge for crate::queue::Edge {
    fn reversed(&self) -> Self {
        Self {
            from: self.to.clone(),
            to: self.from.clone(),
            kind: self.kind,
        }
    }
}

impl IndexedTraversableEdge for crate::server_routes::Edge {
    fn reversed(&self) -> Self {
        Self {
            from: self.to.clone(),
            to: self.from.clone(),
            kind: self.kind,
        }
    }
}
