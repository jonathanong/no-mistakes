use super::*;

fn fixture_graph(selectors: bool) -> (tempfile::TempDir, DepGraph) {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/graph/symbol-import-selectors");
    let dir = crate::test_support::materialize_saved_fixture(&source);
    crate::test_support::git_init(dir.path());
    crate::test_support::git_add_all(dir.path());
    let config = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![("@app/*".to_string(), vec!["./app/*".to_string()])],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    let graph = DepGraph::build_with_plan(
        dir.path(),
        &config,
        GraphBuildPlan {
            symbols: true,
            symbol_import_selectors: selectors,
            ..Default::default()
        },
    )
    .unwrap();
    (dir, graph)
}

fn has_edge(graph: &DepGraph, from: NodeId, to: NodeId, kind: EdgeKind) -> bool {
    graph
        .dependencies_of_node(&from)
        .into_iter()
        .flatten()
        .any(|(target, edge)| target == &to && edge == &kind)
}

#[test]
fn syntactic_selectors_include_unused_imports_and_type_alias_relays() {
    let (dir, graph) = fixture_graph(true);
    let file = |name| NodeId::file(dir.path().join(name));
    let symbol = |name, export| NodeId::symbol(dir.path().join(name), export);
    for name in ["other", "default"] {
        assert!(has_edge(
            &graph,
            file("consumer.mts"),
            symbol("source.mts", name),
            EdgeKind::SymbolImport
        ));
    }
    assert!(has_edge(
        &graph,
        file("consumer.mts"),
        symbol("barrel.mts", "Contract"),
        EdgeKind::SymbolTypeImport
    ));
    assert!(has_edge(
        &graph,
        symbol("barrel.mts", "Contract"),
        symbol("source.mts", "Contract"),
        EdgeKind::SymbolTypeReexport
    ));
    assert!(has_edge(
        &graph,
        symbol("barrel.mts", "renamed"),
        symbol("alias.mts", "renamed"),
        EdgeKind::SymbolReexport
    ));
    assert!(has_edge(
        &graph,
        symbol("alias.mts", "renamed"),
        symbol("source.mts", "wanted"),
        EdgeKind::SymbolReexport
    ));
    // A local import/export alias relays usage; it does not consume the export.
    assert!(!has_edge(
        &graph,
        file("alias.mts"),
        symbol("source.mts", "wanted"),
        EdgeKind::SymbolImport
    ));
}

#[test]
fn unresolved_selectors_remain_explicit_and_bounded_namespace_edges_are_retained() {
    let (dir, graph) = fixture_graph(true);
    let from = NodeId::file(dir.path().join("unknown.mts"));
    let edges = graph.dependencies_of_node(&from).unwrap();
    let unknowns: Vec<_> = edges
        .iter()
        .filter(|(_, kind)| *kind == EdgeKind::SymbolImportUnknown)
        .collect();
    for suffix in [
        "ambiguous.mts#wanted",
        "ambiguous.mts#default",
        "source.mts#absent",
        "./missing.mjs",
    ] {
        assert!(
            unknowns
                .iter()
                .any(|(node, _)| matches!(node, NodeId::Module(name) if name.ends_with(suffix))),
            "{suffix}: {unknowns:?}"
        );
    }
    assert!(has_edge(
        &graph,
        from,
        NodeId::file(dir.path().join("source.mts")),
        EdgeKind::SymbolImportUnknown
    ));
}

#[test]
fn selector_edges_are_absent_from_legacy_graph_plans() {
    assert!(!GraphBuildPlan::all().symbol_import_selectors);
    let (_dir, graph) = fixture_graph(false);
    for file in graph.all_files() {
        assert!(graph
            .dependencies_of_node(file)
            .into_iter()
            .flatten()
            .all(|(_, kind)| !matches!(
                kind,
                EdgeKind::SymbolImport | EdgeKind::SymbolReexport | EdgeKind::SymbolImportUnknown
            )));
    }
}

#[test]
fn type_import_references_are_unknown_but_assets_are_outside_the_export_catalog() {
    let (dir, graph) = fixture_graph(true);
    assert!(has_edge(
        &graph,
        NodeId::file(dir.path().join("type-reference.mts")),
        NodeId::file(dir.path().join("source.mts")),
        EdgeKind::SymbolImportUnknown
    ));
    // `import('x').Name` is a proven symbol consumer, not an unknown selector.
    let qualified = NodeId::file(dir.path().join("type-qualified.mts"));
    assert!(has_edge(
        &graph,
        qualified.clone(),
        NodeId::symbol(dir.path().join("source.mts"), "Contract"),
        EdgeKind::SymbolTypeImport
    ));
    assert!(graph
        .dependencies_of_node(&qualified)
        .into_iter()
        .flatten()
        .all(|(_, kind)| *kind != EdgeKind::SymbolImportUnknown));
    assert!(graph
        .dependencies_of_node(&NodeId::file(dir.path().join("asset.mts")))
        .into_iter()
        .flatten()
        .all(|(_, kind)| *kind != EdgeKind::SymbolImportUnknown));
}

#[test]
fn missing_configured_aliases_are_unknown_but_external_packages_are_not() {
    let (dir, graph) = fixture_graph(true);
    let from = NodeId::file(dir.path().join("aliases.mts"));
    let edges = graph.dependencies_of_node(&from).unwrap();
    assert!(edges
        .iter()
        .any(|(target, kind)| *kind == EdgeKind::SymbolImportUnknown
            && matches!(target, NodeId::Module(name) if &**name == "@app/missing")));
    assert!(edges
        .iter()
        .all(|(target, kind)| *kind != EdgeKind::SymbolImportUnknown
            || !matches!(target, NodeId::Module(name) if &**name == "unrelated-external-package")));
}

#[test]
fn namespace_reexport_relays_and_missing_named_reexports_preserve_uncertainty() {
    let (dir, graph) = fixture_graph(true);
    for barrel in ["namespace-direct.mts", "namespace-local.mts"] {
        assert!(has_edge(
            &graph,
            NodeId::file(dir.path().join(barrel)),
            NodeId::file(dir.path().join("source.mts")),
            EdgeKind::SymbolImportUnknown
        ));
        assert!(has_edge(
            &graph,
            NodeId::file(dir.path().join("namespace-consumer.mts")),
            NodeId::symbol(dir.path().join(barrel), "namespace"),
            EdgeKind::SymbolImport
        ));
    }
    let missing = NodeId::file(dir.path().join("missing-relay.mts"));
    assert!(graph
        .dependencies_of_node(&missing)
        .into_iter()
        .flatten()
        .any(|(target, kind)| {
            *kind == EdgeKind::SymbolImportUnknown
                && matches!(target, NodeId::Module(name) if name.ends_with("source.mts#absent"))
        }));
    assert!(has_edge(
        &graph,
        NodeId::symbol(dir.path().join("missing-relay.mts"), "renamedAbsent"),
        NodeId::symbol(dir.path().join("source.mts"), "absent"),
        EdgeKind::SymbolReexport
    ));
}

#[test]
fn semantic_type_import_records_distinguish_multiline_declarations_and_same_line_references() {
    let (dir, graph) = fixture_graph(true);
    let source = dir.path().join("source.mts");
    let multiline = NodeId::file(dir.path().join("multiline-type.mts"));
    assert!(has_edge(
        &graph,
        multiline.clone(),
        NodeId::symbol(&source, "Contract"),
        EdgeKind::SymbolTypeImport
    ));
    assert!(graph
        .dependencies_of_node(&multiline)
        .into_iter()
        .flatten()
        .all(|(_, kind)| *kind != EdgeKind::SymbolImportUnknown));
    let mixed = NodeId::file(dir.path().join("mixed-type.mts"));
    assert!(has_edge(
        &graph,
        mixed.clone(),
        NodeId::symbol(&source, "Contract"),
        EdgeKind::SymbolTypeImport
    ));
    assert!(has_edge(
        &graph,
        mixed,
        NodeId::file(source),
        EdgeKind::SymbolImportUnknown
    ));
}

#[test]
fn every_unknown_selector_edge_retains_its_prepared_source_evidence() {
    let (_dir, graph) = fixture_graph(true);
    let mut reasons = std::collections::BTreeSet::new();
    let unknowns: Vec<_> = graph
        .edges
        .edges()
        .iter()
        .filter(|edge| edge.kind == EdgeKind::SymbolImportUnknown)
        .collect();
    assert!(!unknowns.is_empty());
    for edge in unknowns {
        let importer = edge.from.as_file().unwrap();
        let sites = graph.unknown_selector_sites(importer, &edge.to);
        assert!(!sites.is_empty(), "missing source evidence for {edge:?}");
        for site in sites {
            assert!(
                site.line > 0 && !site.specifier.is_empty() && !site.reason.is_empty(),
                "{site:?}"
            );
            reasons.insert(site.reason);
        }
    }
    for reason in [
        "namespace selector",
        "namespace re-export selector",
        "type import selector",
        "dynamic or require selector",
        "unresolved named selector",
        "unresolved re-export selector",
    ] {
        assert!(
            reasons.contains(reason),
            "missing constructor coverage: {reason}"
        );
    }
}
