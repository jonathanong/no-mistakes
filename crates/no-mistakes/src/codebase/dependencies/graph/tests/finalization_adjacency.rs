use super::super::super::*;
use super::{n, p};

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
