#[test]
fn scoped_dependency_traversal_rejects_file_bridges_before_expansion() {
    let source = n("/repo/source.ts");
    let bridge = n("/repo/ignored/bridge.ts");
    let target = n("/repo/target.ts");
    let symbol = NodeId::Symbol {
        file: p("/repo/owner.ts"),
        symbol: "owned".to_string(),
    };
    let queue = NodeId::QueueJob {
        queue_file: p("/repo/queue.ts"),
        job: "work".to_string(),
    };
    let module = NodeId::Module("external".to_string());
    let mut forward = EdgeMap::new();
    forward.insert(
        source.clone(),
        vec![
            (bridge.clone(), EdgeKind::Import),
            (module.clone(), EdgeKind::Import),
            (symbol.clone(), EdgeKind::Import),
            (queue.clone(), EdgeKind::QueueEnqueue),
        ],
    );
    forward.insert(bridge, vec![(target.clone(), EdgeKind::Import)]);
    let graph = from_typed_maps(p("/repo"), forward, EdgeMap::new());
    let universe = [
        p("/repo/source.ts"),
        p("/repo/target.ts"),
        p("/repo/owner.ts"),
        p("/repo/queue.ts"),
    ]
    .into_iter()
    .collect();

    let entries = graph.deps_of_in_file_universe(&[source], None, None, &universe);
    let nodes = entries
        .into_iter()
        .map(|entry| entry.node)
        .collect::<HashSet<_>>();

    assert_eq!(nodes, HashSet::from([module, symbol, queue]));
    assert!(!nodes.contains(&target));
    assert!(graph
        .deps_of_in_file_universe(&[n("/repo/ignored/root.ts")], None, None, &universe)
        .is_empty());
}
