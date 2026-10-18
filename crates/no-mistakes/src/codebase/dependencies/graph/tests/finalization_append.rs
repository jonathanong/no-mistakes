use super::super::super::*;
use super::{n, p, with_materialized_vitest_cache};

#[test]
fn empty_selector_append_preserves_materialized_effective_edges() {
    let mut graph = with_materialized_vitest_cache(test_support::from_typed_maps(
        p("/repo"),
        EdgeMap::default(),
        EdgeMap::default(),
    ));
    let edges_before = graph.edges.edges().to_vec();

    graph.append_canonical_edges(Vec::new());

    assert!(graph.vitest_setup_edges_materialized());
    assert_eq!(graph.edges.edges(), edges_before);
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
    let mut appended =
        with_materialized_vitest_cache(test_support::from_typed_maps(p("/repo"), forward, reverse));

    rebuilt.merge_canonical_edges(selector_edges.clone());
    appended.append_canonical_edges(selector_edges);

    assert!(
        !appended.vitest_setup_edges_materialized(),
        "a non-empty append must invalidate the cached effective edge index"
    );
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
