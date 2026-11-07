use super::*;

#[test]
fn value_import_selectors_abstain_when_a_reexport_source_is_incomplete() {
    let (root, files, mut facts) = prepared();
    let path = root.join("alias.ts");
    let original = facts.ts.remove(&path).unwrap();
    let mut ts = original.ts.as_ref().clone();
    ts.value_flow.incomplete_reason = Some("incomplete value facts".into());
    facts.ts.insert(
        path,
        CheckFileFacts {
            ts: Arc::new(ts),
            ..Default::default()
        }
        .into(),
    );
    let graph = build(&root, files, &facts, true);
    assert!(graph
        .imported_value(&root.join("consumer.ts"), "alias")
        .is_none());
    assert!(graph
        .imported_value(&root.join("consumer.ts"), "selected")
        .is_some());
}

#[test]
fn imported_value_query_rejects_colliding_unknown_and_callable_edges() {
    let (root, _, facts) = prepared();
    let consumer = root.join("consumer.ts");
    let from = NodeId::symbol(&consumer, "selected");
    let target = NodeId::symbol(root.join("source.ts"), "TABLES");
    let owner = facts
        .get_ts_facts(&root.join("source.ts"))
        .unwrap()
        .callable_scope_ids
        .iter()
        .find(|(_, name)| name == "run")
        .unwrap()
        .0;
    for extra in [
        (
            NodeId::symbol(root.join("other.ts"), "TABLES"),
            EdgeKind::SymbolImport,
        ),
        (
            NodeId::module("unknown selector"),
            EdgeKind::SymbolImportUnknown,
        ),
        (NodeId::file(root.join("source.ts")), EdgeKind::SymbolImport),
        (
            NodeId::callable(root.join("source.ts"), "run", owner),
            EdgeKind::SymbolImport,
        ),
    ] {
        let graph = test_support::from_typed_maps(
            root.clone(),
            [(
                from.clone(),
                vec![(target.clone(), EdgeKind::SymbolImport), extra],
            )]
            .into_iter()
            .collect(),
            Default::default(),
        );
        assert!(graph.imported_value(&consumer, "selected").is_none());
    }
    let graph = test_support::from_typed_maps(
        root.clone(),
        [(
            from,
            vec![
                (target, EdgeKind::SymbolImport),
                (NodeId::file(root.join("other.ts")), EdgeKind::Import),
            ],
        )]
        .into_iter()
        .collect(),
        Default::default(),
    );
    assert_eq!(
        graph.imported_value(&consumer, "selected"),
        Some((root.join("source.ts"), "TABLES".into()))
    );
}
