#[test]
fn graph_resolver_forwards_deleted_target_candidates_and_visibility_for_scoped_and_legacy_resolvers(
) {
    let root = PathBuf::from("/graph-import-resolver-candidates");
    let importer = root.join("src/entry.ts");
    let visible: crate::fx::PathSet = [importer.clone()].into_iter().collect();
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![("@app/*".to_string(), vec!["src/*".to_string()])],
        paths_dir: root.clone(),
        base_url: None,
    };
    let catalog =
        crate::codebase::ts_resolver::TsConfigCatalog::forced(&root, tsconfig.clone(), None);
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let resolvers = [
        crate::codebase::ts_resolver::ProjectImportResolver::new(
            &tsconfig,
            Some(&catalog),
            &visible,
            None,
            &session,
        ),
        crate::codebase::ts_resolver::ProjectImportResolver::new(
            &tsconfig, None, &visible, None, &session,
        ),
    ];

    for resolver in resolvers {
        assert!(ImportResolution::visible_files(&resolver)
            .is_some_and(|files| files.visible_len() == 1 && files.contains_visible(&importer)));
        assert!(
            ImportResolution::resolution_candidates(&resolver, "@app/deleted", &importer)
                .contains(&root.join("src/deleted.ts"))
        );
    }
}

#[test]
fn build_graph_uses_typescript_path_pattern_precedence() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/tsconfig/paths-precedence");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let tsconfig = crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let graph = build_graph(&root, &tsconfig);
    let entry = NodeId::file(root.join("src/entry.ts"));
    let deps = graph.deps_of(&[entry], None, None);
    let dependency_paths: crate::fx::PathSet = deps
        .iter()
        .filter_map(|dependency| dependency.node.as_file().map(Path::to_path_buf))
        .collect();

    assert!(dependency_paths.contains(&root.join("src/longest-prefix/pecific/detail.ts")));
    assert!(dependency_paths.contains(&root.join("src/exact.ts")));
    assert!(dependency_paths.contains(&root.join("src/first-tie/value/detail.ts")));
    assert!(dependency_paths.contains(&root.join("src/replacement/value.ts")));
    assert!(!dependency_paths.contains(&root.join("src/catch-all/shadowed/value.ts")));
    assert!(!dependency_paths.contains(&root.join("src/total-length/specific.ts")));
    assert!(!dependency_paths.contains(&root.join("src/wildcard/value.ts")));
    assert!(!dependency_paths.contains(&root.join("src/second-tie/value.ts")));
}
