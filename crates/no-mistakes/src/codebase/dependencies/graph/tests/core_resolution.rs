#[test]
fn graph_resolver_forwards_deleted_target_candidates_for_scoped_and_legacy_resolvers() {
    let root = PathBuf::from("/graph-import-resolver-candidates");
    let importer = root.join("src/entry.ts");
    let visible = HashSet::from([importer.clone()]);
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![("@app/*".to_string(), vec!["src/*".to_string()])],
        paths_dir: root.clone(),
        base_url: None,
    };
    let catalog = crate::codebase::ts_resolver::TsConfigCatalog::forced(
        &root,
        tsconfig.clone(),
        None,
    );
    let resolvers = [
        GraphImportResolver::Scoped(
            crate::codebase::ts_resolver::ScopedImportResolver::from_visible(&catalog, &visible),
        ),
        GraphImportResolver::Legacy(ImportResolver::new(&tsconfig).with_visible(&visible)),
    ];

    for resolver in resolvers {
        assert!(ImportResolution::resolution_candidates(&resolver, "@app/deleted", &importer)
            .contains(&root.join("src/deleted.ts")));
    }
}
