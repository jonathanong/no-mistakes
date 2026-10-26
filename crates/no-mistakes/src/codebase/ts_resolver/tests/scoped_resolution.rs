use super::*;

#[test]
fn scoped_catalog_selects_and_resolves_workspace_aliases() {
    let root = workspace_tsconfig_fixture();
    let web = root.join("apps/web");
    let visible = [
        root.join("tsconfig.json"),
        root.join("tsconfig.base.json"),
        web.join("tsconfig.json"),
        web.join("src/entry.ts"),
        web.join("src/runtime/value.ts"),
        root.join("packages/shared/tsconfig.json"),
        root.join("packages/shared/src/message.ts"),
    ];
    let catalog = TsConfigCatalog::from_visible(&root, &[root.clone(), web.clone()], &visible);
    let importer = web.join("src/entry.ts");
    assert_eq!(
        catalog.provenance_for(&importer).config.as_deref(),
        Some(web.join("tsconfig.json").as_path())
    );
    let visible = visible.into_iter().collect::<crate::fx::PathSet>();
    let resolver = ScopedImportResolver::new(&catalog, &visible);
    assert_eq!(
        resolver.resolve("@runtime/value", &importer),
        Some(web.join("src/runtime/value.ts"))
    );
    assert_eq!(
        resolver.resolve("@shared/message", &importer),
        Some(root.join("packages/shared/src/message.ts"))
    );
    assert_eq!(
        resolver.resolve("@runtime/value", &web.join("src/../src/entry.ts")),
        Some(web.join("src/runtime/value.ts"))
    );
    assert_eq!(resolver.importer_selection_build_count(), 1);

    let outside_importer = root.join("../outside.ts");
    assert_eq!(resolver.resolve("@runtime/value", &outside_importer), None);
    assert_eq!(resolver.resolve("@runtime/value", &outside_importer), None);
    assert_eq!(resolver.importer_selection_build_count(), 2);
}

#[test]
fn scoped_resolution_candidates_use_importer_workspace_aliases_for_deleted_targets() {
    let root = scoped_resolution_candidates_fixture();
    let web = root.join("apps/web");
    let importer = web.join("src/entry.ts");
    let visible = [
        root.join("tsconfig.json"),
        web.join("tsconfig.json"),
        importer.clone(),
    ];
    let catalog = TsConfigCatalog::from_visible(&root, &[root.clone(), web.clone()], &visible);
    let visible = visible.into_iter().collect::<crate::fx::PathSet>();
    let resolver = ScopedImportResolver::new(&catalog, &visible);

    let candidates = resolver.resolution_candidates("@fixture/deleted", &importer);

    assert!(candidates.contains(&web.join("src/workspace/deleted.ts")));
    assert!(!candidates.contains(&root.join("src/root/deleted.ts")));
}

#[test]
fn scoped_import_resolver_facade_classifies_workspace_aliases() {
    let root = workspace_tsconfig_fixture();
    let web = root.join("apps/web");
    let importer = web.join("src/entry.ts");
    let target = web.join("src/runtime/value.ts");
    let visible = [
        root.join("tsconfig.json"),
        root.join("tsconfig.base.json"),
        web.join("tsconfig.json"),
        importer.clone(),
        target.clone(),
    ];
    let catalog = TsConfigCatalog::from_visible(&root, &[root.clone(), web], &visible);
    let visible = visible.into_iter().collect::<crate::fx::PathSet>();
    let resolver = ScopedImportResolver::new(&catalog, &visible);
    let workspace = crate::codebase::workspaces::IndexedWorkspaceMap::default();

    let classification = ImportResolverFacade::classify_import(
        &resolver,
        "@runtime/value",
        &importer,
        &workspace,
        &visible,
    );

    assert_eq!(classification.resolver_path(), Some(target.as_path()));
    assert!(classification.workspace_path().is_none());
}

#[test]
fn scoped_catalog_session_reuses_dynamic_importer_scope_cache() {
    let root = workspace_tsconfig_fixture();
    let web = root.join("apps/web");
    let importer = web.join("src/entry.ts");
    let target = web.join("src/runtime/value.ts");
    let visible = [
        root.join("tsconfig.json"),
        root.join("tsconfig.base.json"),
        web.join("tsconfig.json"),
        importer.clone(),
        target.clone(),
    ];
    let catalog = TsConfigCatalog::from_visible(&root, &[root.clone(), web], &visible);
    let visible = visible.into_iter().collect::<crate::fx::PathSet>();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));

    let first = ScopedImportResolver::new_in_session(&catalog, &visible, &session);
    assert_eq!(
        first.resolve("@runtime/value", &importer),
        Some(target.clone())
    );
    assert_eq!(first.scope_key_build_count(), 1);
    assert_eq!(first.scope_cache_lookup_count(), 1);
    let first_work = observer.snapshot().work;
    let second = ScopedImportResolver::new_in_session(&catalog, &visible, &session);
    assert_eq!(second.resolve("@runtime/value", &importer), Some(target));
    let repeated_work = observer.snapshot().work;

    assert_eq!(
        repeated_work["resolver.computations"],
        first_work["resolver.computations"]
    );
    assert!(
        repeated_work["resolver.cache_hits"]
            > first_work
                .get("resolver.cache_hits")
                .copied()
                .unwrap_or_default()
    );
}

#[test]
fn project_resolver_legacy_path_retains_the_explicit_shared_cache() {
    let root = workspace_tsconfig_fixture();
    let web = root.join("apps/web");
    let importer = web.join("src/entry.ts");
    let target = web.join("src/runtime/value.ts");
    let tsconfig = load_tsconfig(&web.join("tsconfig.json")).unwrap();
    let visible: crate::fx::PathSet = [importer.clone(), target.clone()].into_iter().collect();
    let cache = ImportResolutionCache::default();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));

    let first = ProjectImportResolver::new(&tsconfig, None, &visible, Some(&cache), &session);
    assert!(matches!(&first, ProjectImportResolver::Legacy(_)));
    assert_eq!(
        first.resolve("@runtime/value", &importer),
        Some(target.clone())
    );
    let first_work = observer.snapshot().work;

    let second = ProjectImportResolver::new(&tsconfig, None, &visible, Some(&cache), &session);
    assert_eq!(
        second.resolve("@runtime/value", &importer),
        Some(target),
        "legacy project resolvers reuse their explicit request cache"
    );
    let repeated_work = observer.snapshot().work;

    assert_eq!(
        repeated_work["resolver.computations"],
        first_work["resolver.computations"]
    );
    assert!(
        repeated_work["resolver.cache_hits"]
            > first_work
                .get("resolver.cache_hits")
                .copied()
                .unwrap_or_default()
    );
}

#[test]
fn queue_compatibility_never_clears_the_standard_fixed_session_cache() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/tsconfig-paths/fixture");
    let importer = root.join("enqueue.ts");
    let target = root.join("queues/email.ts");
    let visible_paths = [importer.clone(), target.clone()];
    let visible = visible_paths
        .iter()
        .cloned()
        .collect::<crate::fx::PathSet>();
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root.clone(),
        base_url: None,
    };
    let catalog = TsConfigCatalog::forced(&root, tsconfig, None);
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));

    let standard = ScopedImportResolver::new_in_session(&catalog, &visible, &session);
    assert_eq!(standard.resolve("queues/email", &importer), None);
    let before = observer.snapshot().work;
    let queue = ScopedImportResolver::new_in_session(&catalog, &visible, &session)
        .with_queue_compatibility(&root);
    assert_eq!(
        queue.resolve("queues/email", &importer),
        Some(normalize_path(&target))
    );
    assert_eq!(standard.resolve("queues/email", &importer), None);
    let after = observer.snapshot().work;
    assert_eq!(
        after["resolver.computations"],
        before["resolver.computations"] + 1
    );
    assert!(
        after
            .get("resolver.cache_hits")
            .copied()
            .unwrap_or_default()
            > before
                .get("resolver.cache_hits")
                .copied()
                .unwrap_or_default()
    );
}

#[test]
fn standalone_scoped_resolver_builds_one_visibility_scope_per_selected_catalog_config() {
    let root = workspace_tsconfig_fixture();
    let web = root.join("apps/web");
    let worker = root.join("services/worker");
    let web_target = web.join("src/runtime/value.ts");
    let worker_target = worker.join("src/runtime/value.ts");
    let mut visible = [
        root.join("tsconfig.json"),
        root.join("tsconfig.base.json"),
        web.join("tsconfig.json"),
        web_target.clone(),
        worker.join("tsconfig.worker.json"),
        worker_target.clone(),
    ]
    .into_iter()
    .collect::<crate::fx::PathSet>();
    // The resolver must not sort this request-wide universe for each of the
    // synthetic importers below. They model a large one-config package.
    visible.extend((0..256).map(|index| root.join(format!("generated/{index}.ts"))));
    let catalog = TsConfigCatalog::from_visible(
        &root,
        &[root.clone(), web.clone(), worker.clone()],
        &visible.iter().cloned().collect::<Vec<_>>(),
    );
    let resolver = ScopedImportResolver::new(&catalog, &visible);

    for index in 0..64 {
        let importer = web.join(format!("src/generated-{index}.ts"));
        assert_eq!(
            resolver.resolve("@runtime/value", &importer),
            Some(web_target.clone())
        );
    }
    assert_eq!(resolver.scope_key_build_count(), 1);
    assert_eq!(resolver.scope_cache_lookup_count(), 1);
    assert_eq!(resolver.scope_key_count(), 1);

    // A distinct catalog config with the same alias must retain an isolated
    // result cache and therefore owns a second visibility scope key.
    let worker_importer = worker.join("src/entry.ts");
    assert_eq!(
        resolver.resolve("@runtime/value", &worker_importer),
        Some(worker_target)
    );
    assert_eq!(resolver.scope_key_build_count(), 2);
    assert_eq!(resolver.scope_cache_lookup_count(), 2);
    assert_eq!(resolver.scope_key_count(), 2);
}

#[test]
fn automatic_root_catalog_reuses_one_resolver_without_becoming_forced() {
    let root = fixed_root_tsconfig_fixture();
    let config = root.join("tsconfig.json");
    let entry = root.join("src/entry.ts");
    let value = root.join("src/value.ts");
    let package = root.join("packages/without-tsconfig/package.json");
    let external = root.join("src/external.ts");
    let visible = vec![
        config.clone(),
        entry.clone(),
        value.clone(),
        package,
        external.clone(),
    ];
    let catalog = TsConfigCatalog::from_visible(&root, std::slice::from_ref(&root), &visible);

    let provenance = catalog.provenance_for(&entry);
    assert_eq!(
        provenance.config,
        Some(normalize_path(&config.canonicalize().unwrap()))
    );
    assert!(!provenance.forced);
    assert_eq!(catalog.fixed_config(), Some(catalog.config_for(&entry)));
    assert!(catalog.provenance_for(&external).config.is_none());

    let visible = visible.into_iter().collect::<crate::fx::PathSet>();
    let resolver = ScopedImportResolver::new(&catalog, &visible);
    assert!(resolver.uses_fixed_resolver());
    assert_eq!(
        resolver.resolve("@root/value", &entry),
        Some(normalize_path(&value.canonicalize().unwrap()))
    );
    assert_eq!(resolver.resolve("@root/value", &external), None);
}
