use super::*;

#[test]
fn graph_classification_reuses_the_exact_domains_session_import_table() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/tsconfig/workspace-resolution")
        .canonicalize()
        .unwrap();
    let importer = root.join("apps/web/src/entry.ts");
    let target = root.join("apps/web/src/runtime/value.ts");
    let visible = crate::codebase::ts_source::discover_visible_paths(&root);
    let catalog =
        TsConfigCatalog::from_visible(&root, &[root.clone(), root.join("apps/web")], &visible);
    let visible = visible.into_iter().collect::<crate::fx::PathSet>();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));
    let domain = ScopedImportResolver::new_in_session(&catalog, &visible, &session);
    assert_eq!(
        domain.resolve("@runtime/value", &importer),
        Some(target.clone())
    );
    assert_eq!(domain.resolve("@runtime/missing", &importer), None);
    let graph_cache = ImportResolutionCache::default();
    let graph = ScopedImportResolver::new_in_session(&catalog, &visible, &session)
        .with_shared_cache(&graph_cache);
    let workspace = crate::codebase::workspaces::IndexedWorkspaceMap::default();
    let known = graph.classify_import("@runtime/value", &importer, &workspace, &visible);
    assert_eq!(known.resolver_path(), Some(target.as_path()));
    assert!(graph
        .classify_import("@runtime/missing", &importer, &workspace, &visible)
        .resolver_path()
        .is_none());
    assert_eq!(observer.snapshot().work["resolver.computations"], 2);
    assert_eq!(graph_cache.classification_count(), 2);
    assert_eq!(domain.resolve("@runtime/value", &importer), Some(target));
    assert_eq!(observer.snapshot().work["resolver.computations"], 2);
}

#[test]
fn graph_projection_cannot_clear_a_warmed_legacy_session_scope() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/parser-count/signature-impact")
        .canonicalize()
        .unwrap();
    let importer = root.join("consumer.mts");
    let target = root.join("utils.mts");
    let visible = crate::fx::PathSet::from_iter([importer.clone(), target.clone()]);
    let config = TsConfig {
        dir: root.clone(),
        paths_dir: root,
        ..Default::default()
    };
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));
    let domain = ImportResolver::new_in_session(&config, Some(&visible), &session);
    assert_eq!(
        domain.resolve("./utils.mjs", &importer),
        Some(target.clone())
    );
    let graph_cache = ImportResolutionCache::default();
    let graph = ImportResolver::new_in_session(&config, Some(&visible), &session)
        .with_shared_cache(&graph_cache);
    assert_eq!(
        graph.resolve("./utils.mjs", &importer),
        Some(target.clone())
    );
    assert_eq!(domain.resolve("./utils.mjs", &importer), Some(target));
    assert_eq!(observer.snapshot().work["resolver.computations"], 2);
}

#[test]
fn adaptive_queue_projection_keeps_shared_graph_classification_after_policy_selection() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/tsconfig/workspace-resolution")
        .canonicalize()
        .unwrap();
    let importer = root.join("apps/web/src/entry.ts");
    let target = root.join("apps/web/src/runtime/value.ts");
    let visible = crate::codebase::ts_source::discover_visible_paths(&root);
    let catalog =
        TsConfigCatalog::from_visible(&root, &[root.clone(), root.join("apps/web")], &visible);
    let visible = visible.into_iter().collect::<crate::fx::PathSet>();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));
    let queue = ScopedImportResolver::new_in_session(&catalog, &visible, &session)
        .with_queue_compatibility(&root);
    assert_eq!(
        queue.resolve("@runtime/value", &importer),
        Some(target.clone())
    );
    let graph_cache = ImportResolutionCache::default();
    let graph = queue.with_shared_cache(&graph_cache);
    let workspace = crate::codebase::workspaces::IndexedWorkspaceMap::default();
    assert_eq!(
        graph
            .classify_import("@runtime/value", &importer, &workspace, &visible)
            .resolver_path(),
        Some(target.as_path())
    );
    assert_eq!(graph_cache.classification_count(), 1);
    assert_eq!(observer.snapshot().work["resolver.computations"], 1);
}
