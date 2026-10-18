use super::*;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/parser-count/signature-impact")
        .canonicalize()
        .unwrap()
}

#[test]
fn session_scopes_share_visibility_and_successful_and_failed_imports() {
    let root = fixture();
    let paths = [root.join("consumer.mts"), root.join("utils.mts")];
    let visible: crate::fx::PathSet = paths.iter().cloned().collect();
    let copied: HashSet<_> = paths.into_iter().rev().collect();
    let config = TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let catalog = TsConfigCatalog::forced(&root, config, None);
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));
    let first = ScopedImportResolver::new_in_session(&catalog, &visible, &session);
    let second = ScopedImportResolver::new_in_session(&catalog, &copied, &session);
    // Equal contents from separate domains must keep one prepared identity.
    assert!(std::ptr::eq(
        first.visible.as_ref().unwrap().lookup(),
        second.visible.as_ref().unwrap().lookup()
    ));
    for resolver in [&first, &second] {
        assert_eq!(
            resolver.resolve("./utils.mjs", &root.join("consumer.mts")),
            Some(root.join("utils.mts"))
        );
        assert_eq!(
            resolver.resolve("./absent.mjs", &root.join("consumer.mts")),
            None
        );
    }
    let work = observer.snapshot().work;
    assert_eq!(work["resolver.scope_cache_requests"], 2);
    assert_eq!(work["resolver.visible_normalizations"], 1);
    assert_eq!(work["resolver.visible_canonicalize_calls"], 2);
    assert_eq!(work["resolver.scope_constructions"], 1);
    assert_eq!(work["resolver.computations"], 2);
}

#[test]
fn distinct_visible_projections_keep_negative_results_separate() {
    let root = fixture();
    let importer = root.join("consumer.mts");
    let target = root.join("utils.mts");
    let full = crate::fx::PathSet::from_iter([importer.clone(), target.clone()]);
    let narrow = crate::fx::PathSet::from_iter([importer.clone()]);
    let config = TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let catalog = TsConfigCatalog::forced(&root, config, None);
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));
    let baseline = ScopedImportResolver::new_in_session(&catalog, &narrow, &session);
    assert_eq!(baseline.resolve("./utils.mjs", &importer), None);
    let additive = ScopedImportResolver::new_in_session(&catalog, &full, &session);
    assert_eq!(additive.resolve("./utils.mjs", &importer), Some(target));
    assert_eq!(baseline.resolve("./utils.mjs", &importer), None);
    let work = observer.snapshot().work;
    assert_eq!(work["resolver.scope_constructions"], 2);
    assert_eq!(work["resolver.computations"], 2);
    assert_eq!(work["resolver.visible_canonicalize_calls"], 2);
}

#[test]
fn concurrent_scopes_compute_visibility_and_resolution_once() {
    let root = fixture();
    let visible =
        crate::fx::PathSet::from_iter([root.join("consumer.mts"), root.join("utils.mts")]);
    let config = TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let catalog = TsConfigCatalog::forced(&root, config, None);
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));
    std::thread::scope(|scope| {
        for _ in 0..16 {
            scope.spawn(|| {
                let resolver = ScopedImportResolver::new_in_session(&catalog, &visible, &session);
                assert_eq!(
                    resolver.resolve("./utils.mjs", &root.join("consumer.mts")),
                    Some(root.join("utils.mts"))
                );
            });
        }
    });
    let work = observer.snapshot().work;
    assert_eq!(work["resolver.visible_normalizations"], 1);
    assert_eq!(work["resolver.scope_constructions"], 1);
    assert_eq!(work["resolver.visible_canonicalize_calls"], 2);
    assert_eq!(work["resolver.computations"], 1);
}

#[test]
fn queue_scopes_share_results_only_with_the_same_policy_and_root() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/tsconfig-paths/fixture")
        .canonicalize()
        .unwrap();
    let importer = root.join("enqueue.ts");
    let target = root.join("queues/email.ts");
    let visible = crate::fx::PathSet::from_iter([importer.clone(), target.clone()]);
    let config = TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let catalog = TsConfigCatalog::forced(&root, config, None);
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));
    let standard = ScopedImportResolver::new_in_session(&catalog, &visible, &session);
    assert_eq!(standard.resolve("queues/email", &importer), None);
    let graph_cache = ImportResolutionCache::default();
    let workspace = crate::codebase::workspaces::IndexedWorkspaceMap::default();
    for _ in 0..2 {
        let queue = ScopedImportResolver::new_in_session(&catalog, &visible, &session)
            .with_queue_compatibility(&root);
        assert_eq!(
            queue.resolve("queues/email", &importer),
            Some(target.clone())
        );
        let graph = queue.with_shared_cache(&graph_cache);
        assert_eq!(
            graph
                .classify_import("queues/email", &importer, &workspace, &visible)
                .resolver_path(),
            Some(target.as_path())
        );
    }
    let other_root = root.join("queues");
    let other = ScopedImportResolver::new_in_session(&catalog, &visible, &session)
        .with_queue_compatibility(&other_root);
    assert_eq!(other.resolve("queues/email", &importer), None);
    assert_eq!(standard.resolve("queues/email", &importer), None);
    let work = observer.snapshot().work;
    assert_eq!(work["resolver.scope_constructions"], 3);
    assert_eq!(work["resolver.computations"], 3);
    assert_eq!(work["resolver.visible_normalizations"], 1);
    assert_eq!(graph_cache.classification_count(), 1);
}
