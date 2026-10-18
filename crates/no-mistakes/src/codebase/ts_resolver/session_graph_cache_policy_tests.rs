use super::*;

fn queue_policy_fixture() -> (
    PathBuf,
    PathBuf,
    PathBuf,
    crate::fx::PathSet,
    TsConfigCatalog,
) {
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
    (root, importer, target, visible, catalog)
}

#[test]
fn shared_classification_cache_keeps_standard_and_queue_answers_apart() {
    let (root, importer, target, visible, catalog) = queue_policy_fixture();
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let workspace = crate::codebase::workspaces::IndexedWorkspaceMap::default();
    let cache = ImportResolutionCache::default();
    // One graph cache, two facades. Queue compatibility rewrites `queues/email`
    // to a file; the standard policy must still classify that specifier as external.
    let standard = ScopedImportResolver::new_in_session(&catalog, &visible, &session)
        .with_shared_cache(&cache);
    let queue = ScopedImportResolver::new_in_session(&catalog, &visible, &session)
        .with_queue_compatibility(&root)
        .with_shared_cache(&cache);

    assert_eq!(
        queue
            .classify_import("queues/email", &importer, &workspace, &visible)
            .resolver_path(),
        Some(target.as_path())
    );
    assert_eq!(
        standard
            .classify_import("queues/email", &importer, &workspace, &visible)
            .resolver_path(),
        None
    );
    assert_eq!(
        queue
            .classify_import("queues/email", &importer, &workspace, &visible)
            .resolver_path(),
        Some(target.as_path())
    );
    assert_eq!(standard.resolve("queues/email", &importer), None);
    assert_eq!(
        queue.resolve("queues/email", &importer),
        Some(target.clone())
    );

    let reverse = ImportResolutionCache::default();
    let standard_first = ScopedImportResolver::new_in_session(&catalog, &visible, &session)
        .with_shared_cache(&reverse);
    let queue_second = ScopedImportResolver::new_in_session(&catalog, &visible, &session)
        .with_queue_compatibility(&root)
        .with_shared_cache(&reverse);
    assert_eq!(
        standard_first
            .classify_import("queues/email", &importer, &workspace, &visible)
            .resolver_path(),
        None
    );
    assert_eq!(
        queue_second
            .classify_import("queues/email", &importer, &workspace, &visible)
            .resolver_path(),
        Some(target.as_path())
    );
    assert_eq!(reverse.classification_count(), 2);

    let other_root = root.join("queues");
    let other = ScopedImportResolver::new_in_session(&catalog, &visible, &session)
        .with_queue_compatibility(&other_root)
        .with_shared_cache(&cache);
    assert_eq!(
        other
            .classify_import("queues/email", &importer, &workspace, &visible)
            .resolver_path(),
        None
    );
    assert_eq!(
        queue
            .classify_import("queues/email", &importer, &workspace, &visible)
            .resolver_path(),
        Some(target.as_path())
    );
    assert_eq!(cache.classification_count(), 3);
    assert_eq!(cache.request_count(), 5);
}

#[test]
fn standalone_queue_facade_reuses_one_classification_without_a_session_interner() {
    let (root, importer, target, visible, catalog) = queue_policy_fixture();
    let workspace = crate::codebase::workspaces::IndexedWorkspaceMap::default();
    let cache = ImportResolutionCache::default();
    let queue = ScopedImportResolver::from_visible(&catalog, &visible)
        .with_queue_compatibility(&root)
        .with_shared_cache(&cache);
    assert_eq!(
        queue
            .classify_import("queues/email", &importer, &workspace, &visible)
            .resolver_path(),
        Some(target.as_path())
    );
    assert_eq!(
        queue
            .classify_import("queues/email", &importer, &workspace, &visible)
            .resolver_path(),
        Some(target.as_path())
    );
    assert_eq!(cache.classification_count(), 1);
    assert_eq!(cache.request_count(), 2);
}
