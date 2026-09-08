fn call_relationship_fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/call-reachability/graph-star"),
    )
}

fn call_relationship_context(root: &Path) -> SharedTraversalContext {
    SharedTraversalContext::prepare(
        root.to_path_buf(),
        None,
        None,
        graph::GraphBuildPlan {
            calls: true,
            ..Default::default()
        },
    )
    .expect("call relationship fixture should prepare")
}

fn named_barrel_fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/call-reachability/graph-barrel"),
    )
}

#[test]
fn call_relationship_expands_forward_file_roots_to_callable_scopes() {
    let root = call_relationship_fixture_root();
    let mut shared = call_relationship_context(&root);
    let mut args = traverse_args(root.clone(), vec![PathBuf::from("caller.ts")]);
    args.relationships = vec![RelationshipArg::Call];

    let result = collect_and_filter_entries_shared(&args, Direction::Deps, &root, &mut shared)
        .expect("forward call traversal should succeed");

    assert!(result.entries.iter().any(|entry| {
        entry.node == NodeId::symbol(root.join("source.ts"), "direct") && entry.depth == 2
    }));
    assert!(result.entries.iter().any(|entry| {
        entry.node == NodeId::symbol(root.join("source.ts"), "namespaced") && entry.depth == 1
    }));
}

#[test]
fn call_relationship_expands_reverse_file_roots_to_exported_callables() {
    let root = call_relationship_fixture_root();
    let mut shared = call_relationship_context(&root);
    let mut args = traverse_args(root.clone(), vec![PathBuf::from("source.ts")]);
    args.relationships = vec![RelationshipArg::Call];

    let result = collect_and_filter_entries_shared(&args, Direction::Dependents, &root, &mut shared)
        .expect("reverse call traversal should succeed");

    assert!(
        result.entries.iter().any(|entry| {
            entry.node == NodeId::symbol(root.join("caller.ts"), "run") && entry.depth == 1
        }),
        "reverse call traversal: {:#?}",
        result.entries
    );
    assert!(result
        .entries
        .iter()
        .all(|entry| entry.node != NodeId::symbol(root.join("barrel.ts"), "value")));
}

#[test]
fn reverse_call_file_root_expands_reexported_barrel_symbols() {
    let root = named_barrel_fixture_root();
    let mut shared = call_relationship_context(&root);
    let mut args = traverse_args(root.clone(), vec![PathBuf::from("barrel.ts")]);
    args.relationships = vec![RelationshipArg::Call];

    let result = collect_and_filter_entries_shared(&args, Direction::Dependents, &root, &mut shared)
        .expect("barrel reverse call traversal should succeed");

    assert!(result.entries.iter().any(|entry| {
        entry.node == NodeId::symbol(root.join("caller.ts"), "run") && entry.depth == 1
    }), "barrel reverse call traversal: {:#?}", result.entries);
}

#[test]
fn exported_object_member_body_participates_in_call_traversal() {
    let root = named_barrel_fixture_root();
    let mut shared = call_relationship_context(&root);
    let mut args = traverse_args(root.clone(), vec![PathBuf::from("caller.ts")]);
    args.relationships = vec![RelationshipArg::Call];

    let result = collect_and_filter_entries_shared(&args, Direction::Deps, &root, &mut shared)
        .expect("exported object call traversal should succeed");

    assert!(result.entries.iter().any(|entry| {
        entry.node == NodeId::symbol(root.join("source.ts"), "createProgram")
            && entry.depth == 2
    }), "exported object call traversal: {:#?}", result.entries);
}
