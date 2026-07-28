//! Unstable adapters used only by the checked-in performance harness.
//!
//! Keeping these wrappers behind `test-instrumentation` lets Criterion measure
//! the real in-process aggregate paths without making their internal result
//! types part of the supported Rust API.

use crate::edge_index::{CanonicalEdge, PreparedRelationshipIndex};
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

mod production_graph;
pub use production_graph::{
    append_production_selectors, finalize_production_graph, production_graph_fixture,
    ProductionGraphFixture, ProductionGraphSummary,
};

/// Synthetic prepared relationships with paired typed nodes that collapse to
/// one public edge, matching queue job and route identity collisions.
#[derive(Clone)]
pub struct RelationshipProjectionFixture {
    relationships: PreparedRelationshipIndex<u32, u8>,
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
    assert!(logical_edges > 0, "logical_edges must be nonzero");

    let mut edges = Vec::with_capacity((logical_edges * 2) as usize);
    for edge in 0..logical_edges {
        let base = edge * 4;
        edges.extend([
            CanonicalEdge::new(base, base + 2, 1),
            CanonicalEdge::new(base + 1, base + 3, 1),
        ]);
    }
    RelationshipProjectionFixture {
        relationships: PreparedRelationshipIndex::from_edges(edges, relationship_node_name),
        roots: (0..logical_edges)
            .map(|edge| format!("producer-{edge}"))
            .collect(),
    }
}

/// Run the same one-hop public-edge projection used by prepared queue and
/// server edge views.
pub fn project_relationship_edges(
    fixture: &RelationshipProjectionFixture,
) -> RelationshipProjectionSummary {
    let projected_edges = fixture
        .relationships
        .edge_view(&fixture.roots, Some(1), |edge| ProjectedRelationshipEdge {
            from: relationship_node_name(&edge.from),
            to: relationship_node_name(&edge.to),
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

/// Synthetic, deterministic adjacency input for measuring graph finalization.
///
/// This remains behind `test-instrumentation`: it deliberately exposes neither
/// the internal edge index nor a supported programmatic API.
#[derive(Clone)]
pub struct HighFanoutFinalizationFixture {
    forward: HashMap<u32, Vec<(u32, u8)>>,
    reverse: HashMap<u32, Vec<(u32, u8)>>,
}

/// Small stable summary of a finalized synthetic graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighFanoutFinalizationSummary {
    pub canonical_edges: usize,
    pub forward_nodes: usize,
    pub reverse_nodes: usize,
}

/// Create a large, duplicate-containing, high-fanout adjacency fixture.
///
/// The fixture is intentionally independent of repository files so Criterion
/// measures finalization deterministically. Each logical edge is emitted twice
/// in the input and must therefore survive as exactly one canonical edge.
pub fn high_fanout_finalization_fixture(
    node_count: u32,
    fanout: u32,
) -> HighFanoutFinalizationFixture {
    assert!(
        node_count > fanout,
        "fanout must be smaller than node_count"
    );
    assert!(fanout > 0, "fanout must be nonzero");

    let mut forward = HashMap::with_capacity(node_count as usize);
    let mut reverse = HashMap::with_capacity(node_count as usize);
    for source in 0..node_count {
        let mut adjacent = Vec::with_capacity((fanout * 2) as usize);
        for offset in 1..=fanout {
            let target = (source + offset) % node_count;
            let kind = (offset % 3) as u8;
            adjacent.push((target, kind));
            adjacent.push((target, kind));
            reverse
                .entry(target)
                .or_insert_with(Vec::new)
                .extend([(source, kind), (source, kind)]);
        }
        forward.insert(source, adjacent);
    }
    HighFanoutFinalizationFixture { forward, reverse }
}

/// Normalize duplicate adjacency and build the same source-ordered canonical
/// edge index used by graph finalization.
pub fn finalize_high_fanout_adjacency(
    mut fixture: HighFanoutFinalizationFixture,
) -> HighFanoutFinalizationSummary {
    for adjacent in fixture
        .forward
        .values_mut()
        .chain(fixture.reverse.values_mut())
    {
        adjacent.sort_unstable();
        adjacent.dedup();
    }
    let index = crate::edge_index::EdgeIndex::from_normalized_adjacency_maps_by_source(
        fixture.forward,
        fixture.reverse,
        Ord::cmp,
    );
    HighFanoutFinalizationSummary {
        canonical_edges: index.edges().len(),
        forward_nodes: index.forward().len(),
        reverse_nodes: index.reverse().len(),
    }
}

/// Return canonical edge order for deterministic benchmark fixture tests.
pub fn high_fanout_finalization_signature(
    mut fixture: HighFanoutFinalizationFixture,
) -> Vec<(u32, u32, u8)> {
    for adjacent in fixture
        .forward
        .values_mut()
        .chain(fixture.reverse.values_mut())
    {
        adjacent.sort_unstable();
        adjacent.dedup();
    }
    crate::edge_index::EdgeIndex::from_normalized_adjacency_maps_by_source(
        fixture.forward,
        fixture.reverse,
        Ord::cmp,
    )
    .edges()
    .iter()
    .map(|edge| (edge.from, edge.to, edge.kind))
    .collect()
}

/// Run every configured `check` domain and serialize the stable public report.
pub fn check_json(root: &Path) -> Result<String> {
    crate::ast::with_request_parse_cache(|| {
        let results = crate::check_runner::run_all(root.to_path_buf(), None, None)?;
        Ok(serde_json::to_string(&crate::check_runner::json_value(
            &results,
        ))?)
    })
}

/// Run the aggregate check with a scoped observer and return its internal
/// diagnostics without writing stderr.
pub fn check_json_observed(
    root: &Path,
    verbose: bool,
) -> Result<(String, crate::diagnostics::DiagnosticsSnapshot)> {
    let observer = crate::diagnostics::InvocationObserver::new(verbose);
    let output = {
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        check_json(root)?
    };
    Ok((output, observer.snapshot()))
}

/// Run the same multi-report engine used by the asynchronous N-API task.
pub fn analyze_project_json(options_json: String) -> napi::Result<String> {
    crate::ast::with_request_parse_cache(|| {
        crate::napi_api::analyze_project_json_impl(options_json)
    })
}

/// Run the multi-report engine with an explicitly scoped observer. The N-API
/// response remains identical; diagnostics are returned only to the harness.
pub fn analyze_project_json_observed(
    options_json: String,
) -> napi::Result<(String, crate::diagnostics::DiagnosticsSnapshot)> {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let output = {
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        analyze_project_json(options_json)?
    };
    Ok((output, observer.snapshot()))
}

#[cfg(test)]
mod tests;
