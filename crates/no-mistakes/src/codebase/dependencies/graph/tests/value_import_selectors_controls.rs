use super::*;

fn incomplete_reexport() -> (PathBuf, Vec<PathBuf>, CheckFactMap) {
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
    (root, files, facts)
}

#[test]
fn value_import_selectors_abstain_when_a_reexport_source_is_incomplete() {
    let (root, files, facts) = incomplete_reexport();
    let tsconfig = load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let graph_files = GraphFiles::from_files(files);
    let mut inputs = call_resolution_inputs(&root, &tsconfig, &graph_files);
    inputs.plan = GraphBuildPlan {
        calls: true,
        imports: true,
        value_import_selectors: true,
        ..Default::default()
    };
    let resolver = ImportResolver::new(&tsconfig);
    let indexes = CallableResolutionIndexes::default();
    let names =
        populate_callable_export_resolutions(&inputs, &facts, &resolver, &indexes, &mut fx_map())
            .unwrap();
    let edges = project_value_import_selectors(&inputs, &facts, &resolver, &indexes, &names);
    let consumer = root.join("consumer.ts");
    assert!(!edges
        .iter()
        .any(|(from, _, _)| from == &NodeId::symbol(&consumer, "alias")));
    assert!(edges.contains(&(
        NodeId::symbol(&consumer, "selected"),
        NodeId::symbol(root.join("source.ts"), "TABLES"),
        EdgeKind::SymbolImport,
    )));
}

#[test]
fn value_import_selector_demand_preserves_incomplete_prepared_graph_failure() {
    let (root, files, facts) = incomplete_reexport();
    let tsconfig = load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let errors: Vec<_> = [false, true]
        .into_iter()
        .map(|value_import_selectors| {
            DepGraph::build_with_plan_file_list_and_check_facts(
                &root,
                &tsconfig,
                GraphBuildPlan {
                    calls: true,
                    imports: true,
                    value_import_selectors,
                    ..Default::default()
                },
                files.clone(),
                &facts,
            )
            .err()
            .expect("incomplete prepared facts must fail before graph projection")
            .to_string()
        })
        .collect();
    assert_eq!(errors[0], errors[1]);
    assert!(errors[0].contains("alias.ts: incomplete value facts"));
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
