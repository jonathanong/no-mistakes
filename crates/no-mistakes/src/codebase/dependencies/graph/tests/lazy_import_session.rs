#[test]
fn lazy_import_facts_memoize_parse_errors() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../fixtures/codebase/dependencies/selector-malformed-app-source/fixture",
        ),
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
        visible: [malformed.clone()].into(),
        canonical_visible: HashMap::new(),
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
