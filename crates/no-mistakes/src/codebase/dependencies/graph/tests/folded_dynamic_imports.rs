use super::*;

fn fixture_graph(selectors: bool) -> (tempfile::TempDir, DepGraph) {
    fixture_graph_named("folded-dynamic-graph", selectors)
}

fn fixture_graph_named(name: &str, selectors: bool) -> (tempfile::TempDir, DepGraph) {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/production-exports-only-used-by-tests/fixture")
        .join(name);
    let dir = crate::test_support::materialize_saved_fixture(&source);
    crate::test_support::git_init(dir.path());
    crate::test_support::git_add_all(dir.path());
    let graph = DepGraph::build_with_plan(
        dir.path(),
        &TsConfig {
            dir: dir.path().to_path_buf(),
            paths: Vec::new(),
            paths_dir: dir.path().to_path_buf(),
            base_url: None,
        },
        GraphBuildPlan {
            imports: true,
            symbols: true,
            symbol_import_selectors: selectors,
            ..Default::default()
        },
    )
    .unwrap();
    (dir, graph)
}

#[test]
fn folded_dynamic_mocks_preserve_isolation_and_same_line_runtime_imports() {
    let (dir, graph) = fixture_graph_named("folded-mock-imports", true);
    let edges = |name: &str| {
        graph
            .dependencies_of_node(&NodeId::file(dir.path().join(name)))
            .unwrap()
    };
    assert!(
        !edges("other/isolated.mts").iter().any(|(_, kind)| matches!(
            kind,
            EdgeKind::DynamicImport
                | EdgeKind::ConditionalDynamicImport
                | EdgeKind::SymbolImportUnknown
        ))
    );
    for name in ["runtime", "factory"] {
        assert!(edges(&format!("other/{name}.mts")).contains(&(
            NodeId::file(dir.path().join(format!("src/{name}.mts"))),
            EdgeKind::SymbolImportUnknown,
        )));
    }
    assert!(edges("other/same-line.mts").contains(&(
        NodeId::symbol(dir.path().join("src/same-line.mts"), "selected"),
        EdgeKind::SymbolImport,
    )));
}

#[test]
fn folded_dynamic_edges_are_opt_in_and_preserve_occurrence_uncertainty() {
    let (dir, graph) = fixture_graph(true);
    let edges = |name: &str| {
        graph
            .dependencies_of_node(&NodeId::file(dir.path().join(name)))
            .unwrap()
    };
    for name in ["consumer.mts", "with space/href.mts"] {
        let target_file = if name == "consumer.mts" {
            "target.mts"
        } else {
            "with space/target.mts"
        };
        let target = NodeId::file(dir.path().join(target_file));
        assert!(edges(name).contains(&(target.clone(), EdgeKind::DynamicImport)));
        assert!(edges(name).contains(&(
            NodeId::symbol(dir.path().join(target_file), "selected"),
            EdgeKind::SymbolImport,
        )));
    }
    for name in ["ambiguous.mts", "with space/pathname.mts", "unresolved.mts"] {
        assert!(!edges(name).iter().any(|(_, kind)| matches!(
            kind,
            EdgeKind::DynamicImport | EdgeKind::ConditionalDynamicImport
        )));
        assert!(edges(name)
            .iter()
            .any(|(target, kind)| matches!(target, NodeId::Module(_))
                && *kind == EdgeKind::SymbolImportUnknown));
    }
    let (off_dir, off) = fixture_graph(false);
    for name in [
        "consumer.mts",
        "with space/href.mts",
        "ambiguous.mts",
        "with space/pathname.mts",
        "unresolved.mts",
    ] {
        assert!(!off
            .dependencies_of_node(&NodeId::file(off_dir.path().join(name)))
            .into_iter()
            .flatten()
            .any(|(_, kind)| matches!(
                kind,
                EdgeKind::DynamicImport
                    | EdgeKind::ConditionalDynamicImport
                    | EdgeKind::SymbolImportUnknown
            )));
    }
}
