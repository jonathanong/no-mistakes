use super::*;

#[test]
fn external_star_exports_follow_public_maps_without_hiding_uncertain_selectors() {
    let (dir, graph) = graph("external-star-exports");
    let consumer = NodeId::file(dir.path().join("src/consumer.mts"));
    let unknown: Vec<_> = graph
        .dependencies_of_node(&consumer)
        .into_iter()
        .flatten()
        .filter(|(_, kind)| *kind == EdgeKind::SymbolImportUnknown)
        .collect();
    assert!(
        unknown.is_empty(),
        "public map exports must resolve: {unknown:#?}"
    );
    for barrel in ["adapter", "typed", "nested"] {
        let symbol = NodeId::symbol(dir.path().join(format!("src/{barrel}.mts")), "Queue");
        assert!(graph
            .dependencies_of_node(&consumer)
            .into_iter()
            .flatten()
            .any(|(target, kind)| target == &symbol && *kind == EdgeKind::SymbolImport));
    }
    let controls = NodeId::file(dir.path().join("src/controls.mts"));
    let unknown: Vec<_> = graph
        .dependencies_of_node(&controls)
        .into_iter()
        .flatten()
        .filter(|(_, kind)| *kind == EdgeKind::SymbolImportUnknown)
        .collect();
    for suffix in [
        "blocked.mts#Queue",
        "missing.mts#Queue",
        "mixed.mts#Queue",
        "adapter.mts#default",
        "adapter.mts#absent",
        "no-map.mts#Queue",
        "unresolved-member.mts#Queue",
    ] {
        assert!(
            unknown.iter().any(
                |(target, _)| matches!(target, NodeId::Module(name) if name.ends_with(suffix))
            ),
            "uncertain selector {suffix} must remain advisory: {unknown:#?}"
        );
    }
}

fn graph(name: &str) -> (tempfile::TempDir, DepGraph) {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../test-cases/codebase-analysis/{name}/fixture"));
    let dir = crate::test_support::materialize_saved_fixture(&source);
    crate::test_support::git_init(dir.path());
    crate::test_support::git_add_all(dir.path());
    let config = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![("@blocked/*".to_owned(), vec!["./missing/*".to_owned()])],
        paths_dir: dir.path().to_path_buf(),
        base_url: Some(dir.path().to_path_buf()),
    };
    let graph = DepGraph::build_with_plan(
        dir.path(),
        &config,
        GraphBuildPlan {
            symbols: true,
            symbol_import_selectors: true,
            ..Default::default()
        },
    )
    .unwrap();
    (dir, graph)
}

#[test]
fn external_star_projection_keeps_cycles_conditions_and_collisions_unknown() {
    let (dir, graph) = graph("external-star-exports-controls");
    let consumer = NodeId::file(dir.path().join("src/consumer.mts"));
    let edges = graph.dependencies_of_node(&consumer).unwrap();
    for barrel in [
        "cycle-a",
        "conditional",
        "relay",
        "distinct",
        "alias",
        "malformed",
        "missing-target",
        "known-ambiguous",
        "escaped-adapter",
        "named-missing",
        "named-namespace",
        "named-cycle",
        "named-type",
        "single-collision",
        "nested-collision",
    ] {
        let suffix = format!("{barrel}.mts#Queue");
        assert!(
            edges
                .iter()
                .any(|(target, kind)| *kind == EdgeKind::SymbolImportUnknown
                    && matches!(target, NodeId::Module(name) if name.ends_with(&suffix))),
            "{barrel}: {edges:#?}"
        );
    }
    for (barrel, kind) in [
        ("baseurl-owned", EdgeKind::SymbolImport),
        ("override", EdgeKind::SymbolImport),
        ("same", EdgeKind::SymbolImport),
        ("runtime-nested", EdgeKind::SymbolImport),
        ("runtime-named", EdgeKind::SymbolImport),
        ("typed-named", EdgeKind::SymbolImport),
        ("types", EdgeKind::SymbolTypeImport),
    ] {
        let symbol = NodeId::symbol(dir.path().join(format!("src/{barrel}.mts")), "Queue");
        assert!(edges
            .iter()
            .any(|(target, edge)| target == &symbol && *edge == kind));
        let suffix = format!("{barrel}.mts#Queue");
        assert!(
            !edges
                .iter()
                .any(|(target, edge)| *edge == EdgeKind::SymbolImportUnknown
                    && matches!(target, NodeId::Module(name) if name.ends_with(&suffix))),
            "{barrel}: {edges:#?}"
        );
    }
    let shared = NodeId::symbol(dir.path().join("src/diamond.mts"), "shared");
    assert!(edges
        .iter()
        .any(|(target, kind)| target == &shared && *kind == EdgeKind::SymbolImport));
    assert!(!edges
        .iter()
        .any(|(target, kind)| *kind == EdgeKind::SymbolImportUnknown
            && matches!(target, NodeId::Module(name) if name.ends_with("diamond.mts#shared"))));
    let typed_name = NodeId::symbol(dir.path().join("src/typed-named.mts"), "QueueType");
    assert!(edges
        .iter()
        .any(|(target, kind)| target == &typed_name && *kind == EdgeKind::SymbolTypeImport));
    let runtime_name = NodeId::symbol(dir.path().join("src/runtime-named.mts"), "Queue");
    assert!(graph.dependencies_of_node(&runtime_name).into_iter().flatten().any(|(target, kind)|
        *kind == EdgeKind::SymbolReexport && matches!(target, NodeId::Module(name) if name.replace('\\', "/").ends_with("runtime-named/queue.js#OriginalQueue"))));
    let typed_origin = NodeId::symbol(dir.path().join("src/typed-named.mts"), "QueueType");
    assert!(graph.dependencies_of_node(&typed_origin).into_iter().flatten().any(|(target, kind)|
        *kind == EdgeKind::SymbolTypeReexport && matches!(target, NodeId::Module(name) if name.replace('\\', "/").ends_with("typed-named/queue.d.ts#OriginalQueue"))));
    let escaped = NodeId::symbol(dir.path().join("src/escaped-adapter.mts"), "Queue");
    assert!(!graph
        .dependencies_of_node(&escaped)
        .into_iter()
        .flatten()
        .any(|(target, _)| matches!(target, NodeId::Module(_))));
    let local = NodeId::file(dir.path().join("src/escaped.mjs"));
    let local_symbol = NodeId::symbol(dir.path().join("src/escaped.mjs"), "Queue");
    assert!(graph
        .dependencies_of_node(&local)
        .into_iter()
        .flatten()
        .any(|(target, _)| target == &local_symbol));
    let types = NodeId::file(dir.path().join("src/types.mts"));
    let value = NodeId::symbol(dir.path().join("src/types.mts"), "valueOnly");
    assert!(!graph
        .dependencies_of_node(&types)
        .into_iter()
        .flatten()
        .any(|(target, _)| target == &value));
}

#[test]
fn hidden_existing_baseurl_source_keeps_ownership_over_an_installed_package() {
    let (dir, graph) = graph("external-star-exports-hidden-local");
    assert!(dir.path().join("local-owned.mts").is_file());
    assert!(dir
        .path()
        .join("node_modules/local-owned/package.json")
        .is_file());
    let consumer = NodeId::file(dir.path().join("src/consumer.mts"));
    let edges = graph.dependencies_of_node(&consumer).unwrap();
    assert!(edges
        .iter()
        .any(|(node, kind)| *kind == EdgeKind::SymbolImportUnknown
            && matches!(node, NodeId::Module(name) if name.ends_with("barrel.mts#Queue"))));
    let barrel = NodeId::symbol(dir.path().join("src/barrel.mts"), "Queue");
    assert!(!graph.dependencies_of_node(&barrel).into_iter().flatten().any(|(node, kind)|
        *kind == EdgeKind::SymbolReexport && matches!(node, NodeId::Module(name) if name.replace('\\', "/").contains("node_modules/local-owned/entry.js#Queue"))));
}
