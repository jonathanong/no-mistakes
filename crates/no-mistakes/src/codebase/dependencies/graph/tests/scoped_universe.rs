#[test]
fn scoped_dependency_traversal_rejects_file_bridges_before_expansion() {
    let source = n("/repo/source.ts");
    let bridge = n("/repo/ignored/bridge.ts");
    let target = n("/repo/target.ts");
    let symbol = NodeId::symbol(p("/repo/owner.ts"), "owned");
    let queue = NodeId::queue_job(p("/repo/queue.ts"), "work");
    let module = NodeId::module("external");
    let mut forward = EdgeMap::default();
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
    let graph = from_typed_maps(p("/repo"), forward, EdgeMap::default());
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

#[test]
fn excluded_file_cuts_only_paths_through_that_file() {
    let test = n("/repo/test.ts");
    let mocked = n("/repo/mocked.ts");
    let shared = n("/repo/shared.ts");
    let leaf = n("/repo/leaf.ts");
    let mut forward = EdgeMap::default();
    forward.insert(test.clone(), vec![(mocked.clone(), EdgeKind::Import)]);
    forward.insert(mocked, vec![(shared.clone(), EdgeKind::Import)]);
    forward.insert(shared.clone(), vec![(leaf.clone(), EdgeKind::DynamicImport)]);
    let graph = from_typed_maps(p("/repo"), forward, EdgeMap::default());
    let universe = [
        p("/repo/test.ts"),
        p("/repo/mocked.ts"),
        p("/repo/shared.ts"),
        p("/repo/leaf.ts"),
    ]
    .into_iter()
    .collect();
    let excluded = HashSet::from([p("/repo/mocked.ts")]);
    assert!(graph
        .deps_of_in_file_universe_excluding_files(&[test.clone()], None, &universe, &excluded)
        .is_empty());

    // The shared descendant remains reachable by an independent import.
    let mut forward = EdgeMap::default();
    forward.insert(
        test.clone(),
        vec![
            (n("/repo/mocked.ts"), EdgeKind::Import),
            (shared.clone(), EdgeKind::Import),
        ],
    );
    forward.insert(n("/repo/mocked.ts"), vec![(shared.clone(), EdgeKind::Import)]);
    forward.insert(shared.clone(), vec![(leaf.clone(), EdgeKind::DynamicImport)]);
    let graph = from_typed_maps(p("/repo"), forward, EdgeMap::default());
    let nodes = graph
        .deps_of_in_file_universe_excluding_files(&[test], None, &universe, &excluded)
        .into_iter()
        .map(|entry| entry.node)
        .collect::<HashSet<_>>();
    assert_eq!(nodes, HashSet::from([shared, leaf]));
}
