use super::super::test_support;
use super::super::*;
use std::path::PathBuf;

#[test]
fn edge_maps_and_edge_index_use_fx_hash() {
    let types = include_str!("../types.rs");
    assert!(
        types.contains("type EdgeMap = FxHashMap"),
        "graph EdgeMap must use rustc-hash FxHashMap"
    );
    let index = include_str!("../../../../edge_index.rs");
    assert!(
        index.contains("forward: FxHashMap") && index.contains("reverse: FxHashMap"),
        "EdgeIndex adjacency must use rustc-hash FxHashMap"
    );
}

#[test]
fn bfs_visited_sets_use_fx_hash() {
    let bfs = include_str!("../bfs.rs");
    assert!(
        bfs.contains("let mut visited: FxHashSet<NodeId> = fx_set()"),
        "BFS visited set must use rustc-hash FxHashSet"
    );
    assert!(
        bfs.contains("let mut dynamic_import_files: FxHashSet<NodeId> = fx_set()"),
        "BFS dynamic-import set must use rustc-hash FxHashSet"
    );
    assert!(
        bfs.contains("let root_nodes: FxHashSet<NodeId>"),
        "BFS root-node set must use rustc-hash FxHashSet"
    );
}

fn p(path: &str) -> PathBuf {
    PathBuf::from(path)
}

fn n(path: &str) -> NodeId {
    NodeId::file(p(path))
}

#[test]
fn normalized_adjacency_flatten_matches_global_sort_oracle() {
    // These nodes share the same display sort keys. Keep this shuffled so the
    // oracle protects typed tie-breakers, duplicate removal, and ordinals.
    let source_file = NodeId::file(p("/repo/source#job"));
    let source_symbol = NodeId::symbol(p("/repo/source"), "job");
    let source_queue = NodeId::queue_job(p("/repo/source"), "job");
    let target_file = NodeId::file(p("/repo/target#job"));
    let target_symbol = NodeId::symbol(p("/repo/target"), "job");
    let target_queue = NodeId::queue_job(p("/repo/target"), "job");
    let empty = n("/repo/empty.ts");
    let edges = vec![
        (
            source_queue.clone(),
            target_symbol.clone(),
            EdgeKind::Selector,
        ),
        (source_file.clone(), target_queue.clone(), EdgeKind::Import),
        (
            source_symbol.clone(),
            target_file.clone(),
            EdgeKind::DynamicImport,
        ),
        (
            source_queue.clone(),
            target_symbol.clone(),
            EdgeKind::Selector,
        ),
        (source_file.clone(), target_queue.clone(), EdgeKind::Import),
    ];
    let mut forward = EdgeMap::default();
    let mut reverse = EdgeMap::default();
    forward.insert(empty.clone(), Vec::new());
    for (from, to, kind) in &edges {
        forward
            .entry(from.clone())
            .or_default()
            .push((to.clone(), *kind));
        reverse
            .entry(to.clone())
            .or_default()
            .push((from.clone(), *kind));
    }

    let mut oracle_forward = forward.clone();
    let mut oracle_reverse = reverse.clone();
    sort_adjacency_lists(&mut oracle_forward, &mut oracle_reverse);
    let mut expected = oracle_forward
        .iter()
        .flat_map(|(from, adjacent)| {
            adjacent
                .iter()
                .map(|(to, kind)| CanonicalEdge::new(from.clone(), to.clone(), *kind))
        })
        .collect::<Vec<_>>();
    expected.sort_by(|left, right| {
        (
            node_sort_key(&left.from),
            &left.from,
            node_sort_key(&left.to),
            &left.to,
            left.kind.sort_key(),
        )
            .cmp(&(
                node_sort_key(&right.from),
                &right.from,
                node_sort_key(&right.to),
                &right.to,
                right.kind.sort_key(),
            ))
    });
    expected.dedup();

    let index = edge_index_from_maps(forward, reverse);
    assert_eq!(index.edges(), expected);
    assert_eq!(
        index
            .forward()
            .get(&empty)
            .map(|adj| adj.neighbors.as_slice()),
        Some(&[][..])
    );
    assert!(index.forward().get(&target_file).is_none());
    assert!(index.reverse().contains_key(&target_file));

    let traversal = index.traverse(
        &[source_file, source_symbol, source_queue],
        crate::edge_index::EdgeDirection::Dependencies,
        Some(1),
    );
    assert_eq!(traversal, expected);
}

#[test]
fn direct_selector_append_matches_historical_rebuild_ordinals() {
    let base_source = n("/repo/base.ts");
    let base_target = n("/repo/base-target.ts");
    let selector_source = n("/repo/test.spec.ts");
    let selector_target = n("/repo/component.tsx");
    let mut forward = EdgeMap::default();
    let mut reverse = EdgeMap::default();
    merge_edges(
        &mut forward,
        &mut reverse,
        vec![(base_source.clone(), base_target, EdgeKind::Import)],
    );
    let selector_edges = vec![
        (
            selector_source.clone(),
            selector_target.clone(),
            EdgeKind::Selector,
        ),
        (
            base_source.clone(),
            n("/repo/base-target.ts"),
            EdgeKind::Import,
        ),
    ];
    let mut rebuilt = test_support::from_typed_maps(p("/repo"), forward.clone(), reverse.clone());
    let mut appended = test_support::from_typed_maps(p("/repo"), forward, reverse);

    rebuilt.merge_canonical_edges(selector_edges.clone());
    appended.append_canonical_edges(selector_edges);

    assert_eq!(appended.edges.edges(), rebuilt.edges.edges());
    assert_eq!(appended.edges.forward(), rebuilt.edges.forward());
    assert_eq!(appended.edges.reverse(), rebuilt.edges.reverse());
    assert_eq!(
        appended.edges.traverse(
            &[base_source, selector_source],
            crate::edge_index::EdgeDirection::Dependencies,
            Some(1),
        ),
        rebuilt.edges.traverse(
            &[n("/repo/base.ts"), n("/repo/test.spec.ts")],
            crate::edge_index::EdgeDirection::Dependencies,
            Some(1),
        )
    );
}

#[test]
fn high_fanout_selector_append_matches_historical_rebuild() {
    let source = n("/repo/test.spec.ts");
    let mut forward = EdgeMap::default();
    let mut reverse = EdgeMap::default();
    let base_edges = (0..1_024)
        .map(|index| {
            (
                source.clone(),
                n(&format!("/repo/components/{index}.tsx")),
                EdgeKind::Import,
            )
        })
        .collect::<Vec<_>>();
    merge_edges(&mut forward, &mut reverse, base_edges);
    let mut selector_edges = Vec::with_capacity(3_072);
    for index in 0..1_024 {
        let target = n(&format!("/repo/components/{index}.tsx"));
        // Existing imports and duplicate selectors must not alter the base
        // ordinals or create repeated selector edges.
        selector_edges.push((source.clone(), target.clone(), EdgeKind::Import));
        selector_edges.push((source.clone(), target.clone(), EdgeKind::Selector));
        selector_edges.push((source.clone(), target, EdgeKind::Selector));
    }
    let mut rebuilt = test_support::from_typed_maps(p("/repo"), forward.clone(), reverse.clone());
    let mut appended = test_support::from_typed_maps(p("/repo"), forward, reverse);

    rebuilt.merge_canonical_edges(selector_edges.clone());
    appended.append_canonical_edges(selector_edges);

    assert_eq!(appended.edges.edges(), rebuilt.edges.edges());
    assert_eq!(appended.edges.forward(), rebuilt.edges.forward());
    assert_eq!(appended.edges.reverse(), rebuilt.edges.reverse());
    assert_eq!(appended.edges.edges().len(), 2_048);
}
