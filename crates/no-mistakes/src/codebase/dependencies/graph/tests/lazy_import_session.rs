#[test]
fn lazy_import_facts_memoize_parse_errors() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/dependencies/selector-malformed-app-source/fixture"),
    );
    let malformed = root.join("web/components/save-button.tsx");
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph_files = GraphFiles {
        all: vec![malformed.clone()],
        indexable: vec![malformed.clone()],
        visible: [malformed.clone()].into_iter().collect(),
        canonical_visible: CanonicalVisible::empty(),
        resource_candidates: vec![],
    };
    let context = TsFactContext::new(&root);
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(
        std::sync::Arc::clone(&observer),
    ));
    let resolver = crate::codebase::ts_resolver::ImportResolver::new_in_session(
        &tsconfig,
        Some(&graph_files.visible),
        &session,
    );

    crate::ast::with_request_parse_cache(|| {
        for _ in 0..2 {
            let (neighbors, collected) = import_neighbors(
                &malformed,
                &resolver,
                &crate::codebase::workspaces::IndexedWorkspaceMap::default(),
                &graph_files,
                None,
                LazyImportFacts::new(None, TsFactPlan::imports(), &context),
                &session,
            );

            assert!(neighbors.is_empty());
            assert!(collected.and_then(|facts| facts.parse_error).is_some());
        }
    });

    let work = observer.snapshot().work;
    assert_eq!(work["source.requests"], 2);
    assert_eq!(work["source.reads"], 1);
    assert_eq!(work["source.cache_hits"], 1);
    assert_eq!(work["parse.requests"], 2);
    assert_eq!(work["parse.files"], 1);
    assert_eq!(work["parse.errors"], 1);
}

#[test]
fn lazy_import_session_does_not_parse_files_twice() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/lazy-import/fixture"),
    );
    let entry = root.join("src/a.mts");
    let reached = root.join("src/b.mts");
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph_files = GraphFiles::discover(&root);
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(
        std::sync::Arc::clone(&observer),
    ));
    let _ = session.visible_paths(&root);
    let workspace = crate::codebase::workspaces::load_indexed_from_files(&root, graph_files.all())
        .unwrap_or_default();
    let context = TsFactContext::new(&root);
    let roots = [NodeId::file(&entry)];

    crate::ast::with_request_parse_cache(|| {
        let (first, _) =
            lazy_import_deps_of_with_files_facts_workspace_resolution_cache_and_session(
                LazyImportBuild {
                    roots: &roots,
                    tsconfig: &tsconfig,
                    tsconfig_catalog: None,
                    max_depth: None,
                    graph_files: &graph_files,
                    allowed: None,
                    facts: LazyImportFacts::new(None, TsFactPlan::imports(), &context),
                    workspace: &workspace,
                    import_resolution_cache: None,
                },
                &session,
            );
        assert!(first
            .iter()
            .any(|entry| entry.node.as_file() == Some(reached.as_path())));

        let first_work = observer.snapshot().work;
        let parse_files = first_work["parse.files"];
        assert!(parse_files > 0, "{first_work:#?}");
        assert_eq!(first_work["discovery.roots"], 1, "{first_work:#?}");
        assert_eq!(
            first_work.get("graph.builds").copied().unwrap_or_default(),
            0,
            "{first_work:#?}"
        );

        let (second, _) =
            lazy_import_deps_of_with_files_facts_workspace_resolution_cache_and_session(
                LazyImportBuild {
                    roots: &roots,
                    tsconfig: &tsconfig,
                    tsconfig_catalog: None,
                    max_depth: None,
                    graph_files: &graph_files,
                    allowed: None,
                    facts: LazyImportFacts::new(None, TsFactPlan::imports(), &context),
                    workspace: &workspace,
                    import_resolution_cache: None,
                },
                &session,
            );
        assert_eq!(first, second);

        let work = observer.snapshot().work;
        assert_eq!(work["parse.files"], parse_files, "{work:#?}");
        assert_eq!(work["discovery.roots"], 1, "{work:#?}");
        assert_eq!(
            work.get("graph.builds").copied().unwrap_or_default(),
            0,
            "{work:#?}"
        );
    });
}
