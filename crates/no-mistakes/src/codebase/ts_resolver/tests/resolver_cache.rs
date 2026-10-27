use super::*;

#[test]
fn import_resolver_uses_visible_file_set() {
    let dir = saved_resolver_fixture("import-resolver-uses-visible-file-set");
    let target = dir.path().join("src").join("utils.mts");
    let importer = dir.path().join("src").join("main.mts");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    let visible: crate::fx::PathSet = [target.clone()].into_iter().collect();
    let resolver = ImportResolver::new(&tc).with_visible(&visible);

    assert_eq!(resolver.resolve("./utils", &importer), Some(target));
}

/// Regression test: `with_visible` (used by every `DepGraph` build and by
/// `server_routes`) must not disable the resolve cache. `resolve()`'s cache-hit
/// branch is a no-op when `cache_enabled` is false, so the "reuses/preserves"
/// tests below pass on identical *results* even with caching off — they don't
/// prove memoization happened. This asserts the cache is actually populated.
#[test]
fn import_resolver_with_visible_keeps_cache_enabled() {
    let dir = saved_resolver_fixture("import-resolver-with-visible-keeps-cache-enabled");
    let target = dir.path().join("src").join("utils.mts");
    let importer = dir.path().join("src").join("main.mts");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    let visible: crate::fx::PathSet = [target].into_iter().collect();
    let resolver = ImportResolver::new(&tc).with_visible(&visible);
    assert!(resolver.cache_enabled);

    resolver.resolve("./utils", &importer);

    assert_eq!(resolver.cache.len(), 1);
}

/// Regression test: calling `with_visible` on a resolver that already has
/// cached entries (resolved against the real filesystem, or an earlier
/// `visible` set) must not leak those stale answers into the new visibility
/// scope — `with_visible` consumes and returns `Self`, so a reused resolver's
/// `cache` would otherwise carry answers computed under different visibility.
#[test]
fn import_resolver_with_visible_clears_stale_cache_entries() {
    let dir = saved_resolver_fixture("import-resolver-with-visible-clears-stale-cache-entries");
    let target = normalize_path(&dir.path().join("src").join("utils.mts"));
    let importer = dir.path().join("src").join("main.mts");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tc);
    assert_eq!(resolver.resolve("./utils", &importer), Some(target));

    let visible: crate::fx::PathSet = crate::fx::PathSet::default();
    let resolver = resolver.with_visible(&visible);

    assert!(resolver.resolve("./utils", &importer).is_none());
}

#[test]
fn import_resolver_cache_reuses_present_result() {
    let dir = saved_resolver_fixture("import-resolver-cache-reuses-present-result");
    let target = normalize_path(&dir.path().join("src").join("utils.mts"));
    let importer = dir.path().join("src").join("main.mts");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    let visible: crate::fx::PathSet = [target.clone()].into_iter().collect();
    let resolver = ImportResolver::new(&tc).with_visible(&visible);

    assert_eq!(resolver.resolve("./utils", &importer), Some(target.clone()));
    assert_eq!(resolver.resolve("./utils", &importer), Some(target));
    assert!(resolver.resolve("./utils.mts", &importer).is_some());
    assert!(resolver.resolve("./missing.mts", &importer).is_none());
}

#[test]
fn import_resolver_cache_preserves_missing_result() {
    let dir = saved_resolver_fixture("import-resolver-cache-preserves-missing-result");
    let importer = dir.path().join("src").join("main.mts");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tc);

    assert!(resolver.resolve("./utils", &importer).is_none());
    // Move saved bytes into place only after the missing result has been cached.
    std::fs::rename(
        dir.path().join("utils.mts"),
        dir.path().join("src/utils.mts"),
    )
    .unwrap();
    assert!(resolver.resolve("./utils", &importer).is_none());
}

#[test]
fn import_resolver_reports_exact_cached_work_for_hits_and_misses() {
    let root = fixture("explicit-json");
    let importer = root.join("src/main.mts");
    let target = normalize_path(&root.join("src/data.json"));
    let visible: crate::fx::PathSet = [target.clone()].into_iter().collect();
    let config = TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root,
        base_url: None,
    };
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let resolver =
        ImportResolver::new_observed(&config, Some(observer.clone())).with_visible(&visible);

    assert_eq!(
        resolver.resolve("./data.json", &importer),
        Some(target.clone())
    );
    assert_eq!(resolver.resolve("./data.json", &importer), Some(target));
    assert!(resolver.resolve("./missing.json", &importer).is_none());
    assert!(resolver.resolve("./missing.json", &importer).is_none());

    let work = observer.snapshot().work;
    assert_eq!(work["resolver.requests"], 4);
    assert_eq!(work["resolver.computations"], 2);
    assert_eq!(work["resolver.cache_hits"], 2);
    assert_eq!(work["resolver.resolved"], 1);
    assert_eq!(work["resolver.unresolved"], 1);
}

#[test]
fn import_resolver_single_flights_concurrent_hits_and_misses() {
    let root = fixture("explicit-json");
    let importer = root.join("src/main.mts");
    let target = normalize_path(&root.join("src/data.json"));
    let visible: crate::fx::PathSet = [target.clone()].into_iter().collect();
    let config = TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root,
        base_url: None,
    };
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let resolver =
        ImportResolver::new_observed(&config, Some(observer.clone())).with_visible(&visible);
    let requests_per_key = 32;
    let barrier = std::sync::Barrier::new(requests_per_key * 2);

    std::thread::scope(|scope| {
        for request in 0..requests_per_key * 2 {
            let resolver = &resolver;
            let importer = &importer;
            let target = &target;
            let barrier = &barrier;
            scope.spawn(move || {
                barrier.wait();
                if request % 2 == 0 {
                    assert_eq!(
                        resolver.resolve("./data.json", importer),
                        Some(target.clone())
                    );
                } else {
                    assert!(resolver.resolve("./missing.json", importer).is_none());
                }
            });
        }
    });

    let work = observer.snapshot().work;
    assert_eq!(work["resolver.requests"], 64);
    assert_eq!(work["resolver.computations"], 2);
    assert_eq!(work["resolver.cache_hits"], 62);
    assert_eq!(work["resolver.resolved"], 1);
    assert_eq!(work["resolver.unresolved"], 1);
}

#[test]
fn import_resolver_session_reuses_only_exact_resolution_scopes() {
    let root = fixture("explicit-json");
    let importer = root.join("src/main.mts");
    let target = normalize_path(&root.join("src/data.json"));
    let visible: crate::fx::PathSet = [target.clone()].into_iter().collect();
    let hidden: crate::fx::PathSet = crate::fx::PathSet::default();
    let config = TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root,
        base_url: None,
    };
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));

    let first = ImportResolver::new_in_session(&config, Some(&visible), &session);
    assert_eq!(
        first.resolve("./data.json", &importer),
        Some(target.clone())
    );
    assert!(first.resolve("./missing.json", &importer).is_none());
    let second = ImportResolver::new_in_session(&config, Some(&visible), &session);
    assert_eq!(
        second.resolve("./data.json", &importer),
        Some(target.clone())
    );
    assert!(second.resolve("./missing.json", &importer).is_none());

    let hidden_scope = ImportResolver::new_in_session(&config, Some(&hidden), &session);
    assert!(hidden_scope.resolve("./data.json", &importer).is_none());
    let filesystem_scope = ImportResolver::new_in_session(&config, None, &session);
    assert_eq!(
        filesystem_scope.resolve("./data.json", &importer),
        Some(target)
    );

    let work = observer.snapshot().work;
    assert_eq!(work["resolver.requests"], 6);
    assert_eq!(work["resolver.computations"], 4);
    assert_eq!(work["resolver.unique_keys"], 4);
    assert_eq!(work["resolver.cache_hits"], 2);
    assert_eq!(work["resolver.resolved"], 2);
    assert_eq!(work["resolver.unresolved"], 2);
}
