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
