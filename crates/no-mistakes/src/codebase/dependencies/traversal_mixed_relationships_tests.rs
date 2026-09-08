fn mixed_relationship_fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/mixed-relationship-traversal/fixture"),
    )
}

fn mixed_relationship_ctx<'a>(
    root: &'a Path,
    tsconfig: &'a TsConfig,
    graph_files: &'a graph::GraphFiles,
    allowed: &'a std::collections::HashSet<EdgeKind>,
) -> TraversalCtx<'a> {
    TraversalCtx {
        root,
        tsconfig,
        graph_files,
        build_plan: graph::GraphBuildPlan::all(),
        allowed: Some(allowed),
        symbols: false,
    }
}

#[test]
fn mixed_import_and_call_relationships_retain_original_and_expanded_roots() {
    let root = mixed_relationship_fixture();
    let entry = root.join("src/entry.mts");
    let entrypoints = vec![Entrypoint {
        file: entry.clone(),
        node: NodeId::file(&entry),
        symbol: None,
    }];
    let roots = vec![NodeId::file(&entry)];
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph_files = graph::GraphFiles::discover(&root);
    let allowed = std::collections::HashSet::from([EdgeKind::Import, EdgeKind::Call]);
    let ctx = mixed_relationship_ctx(&root, &tsconfig, &graph_files, &allowed);

    let entries = deps_entries(None, false, &roots, &entrypoints, &ctx).unwrap();
    assert!(entries
        .iter()
        .any(|entry| entry.node.as_file() == Some(root.join("src/imported.mts").as_path())));
    assert!(entries
        .iter()
        .any(|entry| entry.node.as_file() == Some(root.join("src/called.mts").as_path())));
}

#[test]
fn mixed_import_and_call_dependents_follow_reverse_cross_kind_paths() {
    let root = mixed_relationship_fixture();
    let called = root.join("src/called.mts");
    let entrypoints = vec![Entrypoint {
        file: called.clone(),
        node: NodeId::file(&called),
        symbol: Some("called".to_string()),
    }];
    let roots = vec![NodeId::file(&called)];
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph_files = graph::GraphFiles::discover(&root);
    let allowed = std::collections::HashSet::from([EdgeKind::Import, EdgeKind::Call]);
    let ctx = mixed_relationship_ctx(&root, &tsconfig, &graph_files, &allowed);

    let entries = dependents_entries(&entrypoints, &roots, None, &ctx).unwrap();
    assert!(entries
        .iter()
        .any(|entry| entry.node.as_file() == Some(root.join("src/imported.mts").as_path())));
    assert!(entries
        .iter()
        .any(|entry| entry.node.as_file() == Some(root.join("src/entry.mts").as_path())));
}
