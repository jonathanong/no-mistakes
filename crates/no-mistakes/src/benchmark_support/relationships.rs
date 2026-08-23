use crate::edge_index::{CanonicalEdge, PreparedRelationshipIndex};

/// Synthetic prepared relationships with paired typed nodes that collapse to
/// one public edge, matching queue job and route identity collisions.
#[derive(Clone)]
pub struct RelationshipProjectionFixture {
    relationships: PreparedRelationshipIndex<u32, u8>,
    roots: Vec<String>,
}

/// Raw deterministic relationship input used to isolate index construction in
/// Criterion from projection work.
#[derive(Clone)]
pub struct RelationshipConstructionFixture {
    edges: Vec<CanonicalEdge<u32, u8>>,
    roots: Vec<String>,
}

/// Stable count returned by the prepared relationship projection benchmark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationshipProjectionSummary {
    pub projected_edges: usize,
}

/// Build a deterministic high-volume prepared projection fixture.
///
/// Every logical edge has two distinct typed representations with identical
/// public names. This exercises alias expansion plus first-seen HashSet
/// deduplication without relying on repository files.
pub fn relationship_projection_fixture(logical_edges: u32) -> RelationshipProjectionFixture {
    relationship_index_from_fixture(relationship_construction_fixture(logical_edges))
}

/// Build raw deterministic relationship input for an index-construction-only
/// benchmark. Criterion can clone this fixture outside the timed iteration.
pub fn relationship_construction_fixture(logical_edges: u32) -> RelationshipConstructionFixture {
    assert!(logical_edges > 0, "logical_edges must be nonzero");

    let mut edges = Vec::with_capacity((logical_edges * 2) as usize);
    for edge in 0..logical_edges {
        let base = edge * 4;
        edges.extend([
            CanonicalEdge::new(base, base + 2, 1),
            CanonicalEdge::new(base + 1, base + 3, 1),
        ]);
    }
    RelationshipConstructionFixture {
        edges,
        roots: (0..logical_edges)
            .map(|edge| format!("producer-{edge}"))
            .collect(),
    }
}

pub fn relationship_index_from_fixture(
    fixture: RelationshipConstructionFixture,
) -> RelationshipProjectionFixture {
    RelationshipProjectionFixture {
        relationships: PreparedRelationshipIndex::from_edges(fixture.edges, relationship_node_name),
        roots: fixture.roots,
    }
}

pub fn project_relationship_edges(
    fixture: &RelationshipProjectionFixture,
) -> RelationshipProjectionSummary {
    let projected_edges = fixture
        .relationships
        .edge_view(&fixture.roots, Some(1), |edge, from, to| {
            ProjectedRelationshipEdge {
                from: from.to_owned(),
                to: to.to_owned(),
                kind: edge.kind,
            }
        })
        .len();
    RelationshipProjectionSummary { projected_edges }
}

pub fn project_all_relationship_edges(
    fixture: &RelationshipProjectionFixture,
) -> RelationshipProjectionSummary {
    let projected_edges = fixture
        .relationships
        .edge_view(&[], None, |edge, from, to| ProjectedRelationshipEdge {
            from: from.to_owned(),
            to: to.to_owned(),
            kind: edge.kind,
        })
        .len();
    RelationshipProjectionSummary { projected_edges }
}

#[derive(Clone, Eq, PartialEq, Hash)]
struct ProjectedRelationshipEdge {
    from: String,
    to: String,
    kind: u8,
}

fn relationship_node_name(node: &u32) -> String {
    let edge = node / 4;
    if node % 4 < 2 {
        format!("producer-{edge}")
    } else {
        format!("worker-{edge}")
    }
}
