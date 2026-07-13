#[test]
fn workspace_symbol_graph_includes_visible_entry_and_excludes_gitignored_entry() {
    let dir = crate::test_support::materialize_gitignore_fixture("workspace-symbol");
    crate::test_support::git_init(dir.path());
    crate::test_support::git_add_all(dir.path());
    let tsconfig = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: Vec::new(),
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };

    let graph = DepGraph::build_with_plan(
        dir.path(),
        &tsconfig,
        GraphBuildPlan::imports_and_workspace().with_symbols(true),
    )
    .unwrap();
    let execute = NodeId::Symbol {
        file: dir.path().join("packages/app/src/consumer.mts"),
        symbol: "execute".to_string(),
    };
    let ignored_entry = dir
        .path()
        .join("packages/core/generated-output/index.mts");
    let visible_entry = dir.path().join("packages/visible/src/index.mts");
    let visible_run = NodeId::Symbol {
        file: visible_entry.clone(),
        symbol: "visibleRun".to_string(),
    };

    assert!(ignored_entry.exists());
    assert!(graph
        .dependencies_of_node(&execute)
        .into_iter()
        .flatten()
        .any(|(node, kind)| node == &visible_run && *kind == EdgeKind::WorkspaceImport));
    assert!(graph
        .dependencies_of_node(&execute)
        .into_iter()
        .flatten()
        .all(|(node, _)| node.as_file() != Some(ignored_entry.as_path())));
    assert!(graph
        .all_files()
        .all(|node| node.as_file() != Some(ignored_entry.as_path())));

    let symbol_index = SymbolIndex::build_from_root(dir.path(), &tsconfig).unwrap();
    assert!(symbol_index
        .importers_of(&visible_entry, "visibleRun")
        .into_iter()
        .flatten()
        .any(|(importer, local, is_reexport)| {
            importer == &dir.path().join("packages/app/src/consumer.mts")
                && local == "visibleRun"
                && !is_reexport
        }));
    assert!(symbol_index
        .importers_of(&ignored_entry, "hiddenRun")
        .is_none());
}
