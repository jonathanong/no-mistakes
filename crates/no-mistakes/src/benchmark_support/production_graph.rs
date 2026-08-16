use crate::codebase::dependencies::{EdgeKind, NodeId};
use crate::edge_index::CanonicalEdge;
use std::collections::HashMap;
use std::path::PathBuf;

/// Production-shaped adjacency and selector input for the graph hot paths.
#[derive(Clone)]
pub struct ProductionGraphFixture {
    forward: HashMap<NodeId, Vec<(NodeId, EdgeKind)>>,
    reverse: HashMap<NodeId, Vec<(NodeId, EdgeKind)>>,
    selectors: Vec<CanonicalEdge<NodeId, EdgeKind>>,
}

/// Stable counts from production-shaped finalization and selector append.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProductionGraphSummary {
    pub canonical_edges: usize,
    pub selector_appended_edges: usize,
}

/// Create normalized-path `NodeId` inputs that exercise the production graph
/// comparator and high-fanout selector append path without reading a fixture.
pub fn production_graph_fixture(node_count: u32, fanout: u32) -> ProductionGraphFixture {
    assert!(
        node_count > fanout,
        "fanout must be smaller than node_count"
    );
    assert!(fanout > 0, "fanout must be nonzero");

    let nodes = (0..node_count).map(production_node).collect::<Vec<_>>();
    let mut forward = HashMap::with_capacity(node_count as usize);
    let mut reverse = HashMap::with_capacity(node_count as usize);
    let mut selectors = Vec::with_capacity((node_count * fanout * 3) as usize);
    for (source_index, source) in nodes.iter().enumerate() {
        let mut adjacent = Vec::with_capacity((fanout * 2) as usize);
        for offset in 1..=fanout as usize {
            let target = nodes[(source_index + offset) % nodes.len()].clone();
            let kind = match offset % 3 {
                0 => EdgeKind::Import,
                1 => EdgeKind::TypeImport,
                _ => EdgeKind::DynamicImport,
            };
            adjacent.push((target.clone(), kind));
            adjacent.push((target.clone(), kind));
            reverse
                .entry(target.clone())
                .or_insert_with(Vec::new)
                .extend([(source.clone(), kind), (source.clone(), kind)]);
            // Exercise an existing edge plus duplicate selector demand for
            // every source, matching the shape of selector integration.
            selectors.push(CanonicalEdge::new(source.clone(), target.clone(), kind));
            selectors.push(CanonicalEdge::new(
                source.clone(),
                target.clone(),
                EdgeKind::Selector,
            ));
            selectors.push(CanonicalEdge::new(
                source.clone(),
                target,
                EdgeKind::Selector,
            ));
        }
        forward.insert(source.clone(), adjacent);
    }
    ProductionGraphFixture {
        forward,
        reverse,
        selectors,
    }
}

/// Exercise the real `NodeId` finalization path used by `DepGraph`.
pub fn finalize_production_graph(fixture: ProductionGraphFixture) -> ProductionGraphSummary {
    let index = crate::codebase::dependencies::graph::edge_index_from_maps(
        fixture.forward,
        fixture.reverse,
    );
    ProductionGraphSummary {
        canonical_edges: index.edges().len(),
        selector_appended_edges: 0,
    }
}

/// Exercise in-place, high-fanout selector append after production finalization.
pub fn append_production_selectors(fixture: ProductionGraphFixture) -> ProductionGraphSummary {
    let mut index = crate::codebase::dependencies::graph::edge_index_from_maps(
        fixture.forward,
        fixture.reverse,
    );
    let canonical_edges = index.edges().len();
    index.extend_edges_preserving_ordinals(fixture.selectors);
    ProductionGraphSummary {
        canonical_edges,
        selector_appended_edges: index.edges().len() - canonical_edges,
    }
}

fn production_node(index: u32) -> NodeId {
    let path = crate::codebase::ts_resolver::normalize_path(&PathBuf::from(format!(
        "/benchmark/packages/pkg{}/src/item{index}.ts",
        index % 8
    )));
    match index % 3 {
        0 => NodeId::file(path),
        1 => NodeId::symbol(path, "value".to_string()),
        _ => NodeId::queue_job(path, "run".to_string()),
    }
}
