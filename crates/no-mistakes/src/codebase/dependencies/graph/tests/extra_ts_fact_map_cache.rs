// ── shared app-wide Playwright scan cache (TsFactMap) ────────────────────

/// Regression test: graph-only `TsFactMap` lookups must cache
/// `get_or_compute_route_reachable_files` — the ~8s monorepo scan that
/// `CheckFactMap` already memoizes. Asserting on the returned value alone
/// would not prove this; a non-caching implementation returns the same
/// value too. Call count and `Arc::ptr_eq` are the real guards.
#[test]
fn ts_fact_map_get_or_compute_route_reachable_files_caches_across_calls() {
    use crate::codebase::dependencies::graph::TsFactLookup;
    use crate::codebase::ts_source::facts::TsFactMap;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let facts = TsFactMap::new();
    let calls = AtomicUsize::new(0);
    let compute = || -> anyhow::Result<_> {
        calls.fetch_add(1, Ordering::SeqCst);
        Ok(Default::default())
    };

    let first = facts
        .get_or_compute_route_reachable_files(&cache_settings(), &compute)
        .unwrap();
    let second = facts
        .get_or_compute_route_reachable_files(&cache_settings(), &compute)
        .unwrap();
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "a second call must reuse the cached reachability scan, not recompute"
    );
    assert!(
        std::sync::Arc::ptr_eq(&first, &second),
        "cached calls must return the same Arc allocation, not merely an equal value"
    );
}

#[test]
fn ts_fact_map_get_or_compute_route_reachable_files_caches_compute_errors() {
    use crate::codebase::dependencies::graph::TsFactLookup;
    use crate::codebase::ts_source::facts::TsFactMap;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let facts = TsFactMap::new();
    let calls = AtomicUsize::new(0);
    let failing = || -> anyhow::Result<_> {
        calls.fetch_add(1, Ordering::SeqCst);
        anyhow::bail!("route reachability scan failed")
    };

    let first = facts
        .get_or_compute_route_reachable_files(&cache_settings(), &failing)
        .unwrap_err();
    assert!(first.to_string().contains("route reachability scan failed"));
    let second = facts
        .get_or_compute_route_reachable_files(&cache_settings(), &failing)
        .unwrap_err();
    assert!(
        second
            .to_string()
            .contains("route reachability scan failed")
    );
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "a cached error must not trigger a recompute"
    );
}

#[test]
fn ts_fact_map_clone_shares_playwright_scan_caches() {
    use crate::codebase::dependencies::graph::TsFactLookup;
    use crate::codebase::ts_source::facts::TsFactMap;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let facts = TsFactMap::new();
    let cloned = facts.clone();
    assert!(
        std::sync::Arc::ptr_eq(
            &facts.route_reachable_files_cache,
            &cloned.route_reachable_files_cache
        ),
        "Clone of TsFactMap must share the route-reachability DashMap Arc"
    );

    let calls = AtomicUsize::new(0);
    let compute = || -> anyhow::Result<_> {
        calls.fetch_add(1, Ordering::SeqCst);
        Ok(Default::default())
    };
    let first = facts
        .get_or_compute_route_reachable_files(&cache_settings(), &compute)
        .unwrap();
    let second = cloned
        .get_or_compute_route_reachable_files(&cache_settings(), &compute)
        .unwrap();
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "a cloned TsFactMap must reuse the original's cached reachability scan"
    );
    assert!(
        std::sync::Arc::ptr_eq(&first, &second),
        "cloned-map cache hits must return the same Arc allocation"
    );
}

#[test]
fn ts_fact_map_get_or_compute_methods_cache_per_settings_key() {
    use crate::codebase::dependencies::graph::TsFactLookup;
    use crate::codebase::ts_source::facts::TsFactMap;
    use crate::playwright::selectors::AppSelector;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let facts = TsFactMap::new();
    let selector_calls = AtomicUsize::new(0);
    let compute_selectors = || -> anyhow::Result<Vec<AppSelector>> {
        selector_calls.fetch_add(1, Ordering::SeqCst);
        Ok(Vec::new())
    };
    let first = facts
        .get_or_compute_app_selector_occurrences(&cache_settings(), false, &compute_selectors)
        .unwrap();
    let second = facts
        .get_or_compute_app_selector_occurrences(&cache_settings(), false, &compute_selectors)
        .unwrap();
    assert_eq!(selector_calls.load(Ordering::SeqCst), 1);
    assert!(std::sync::Arc::ptr_eq(&first, &second));
    facts
        .get_or_compute_app_selector_occurrences(&cache_settings(), true, &compute_selectors)
        .unwrap();
    assert_eq!(
        selector_calls.load(Ordering::SeqCst),
        2,
        "a different scan_html_ids key must recompute"
    );

    let route_calls = AtomicUsize::new(0);
    let compute_routes = || -> Vec<crate::routes::Route> {
        route_calls.fetch_add(1, Ordering::SeqCst);
        Vec::new()
    };
    let first = facts.get_or_compute_playwright_routes(&cache_settings(), &compute_routes);
    let second = facts.get_or_compute_playwright_routes(&cache_settings(), &compute_routes);
    assert_eq!(route_calls.load(Ordering::SeqCst), 1);
    assert!(std::sync::Arc::ptr_eq(&first, &second));

    let text_calls = AtomicUsize::new(0);
    let compute_text = || -> anyhow::Result<_> {
        text_calls.fetch_add(1, Ordering::SeqCst);
        Ok(Vec::new())
    };
    let first = facts
        .get_or_compute_app_text_targets(&cache_settings(), &compute_text)
        .unwrap();
    let second = facts
        .get_or_compute_app_text_targets(&cache_settings(), &compute_text)
        .unwrap();
    assert_eq!(text_calls.load(Ordering::SeqCst), 1);
    assert!(std::sync::Arc::ptr_eq(&first, &second));
}

/// Settings-only keys would return reachability computed for a previous
/// fact/graph universe after `extend`. Generation is in the cache key so a
/// later `compute` that sees newly reachable files cannot be skipped.
#[test]
fn ts_fact_map_extend_invalidates_playwright_scan_caches() {
    use crate::codebase::dependencies::graph::TsFactLookup;
    use crate::codebase::ts_source::facts::{TsFactMap, TsFileFacts};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let mut facts = TsFactMap::new();
    let calls = AtomicUsize::new(0);
    let compute = || -> anyhow::Result<_> {
        calls.fetch_add(1, Ordering::SeqCst);
        Ok(Default::default())
    };
    let first = facts
        .get_or_compute_route_reachable_files(&cache_settings(), &compute)
        .unwrap();
    facts.extend(TsFactMap::from([(
        PathBuf::from("/repo/added.ts"),
        TsFileFacts::default(),
    )]));
    let second = facts
        .get_or_compute_route_reachable_files(&cache_settings(), &compute)
        .unwrap();
    assert_eq!(
        calls.load(Ordering::SeqCst),
        2,
        "extending the fact universe must recompute route reachability"
    );
    assert!(
        !std::sync::Arc::ptr_eq(&first, &second),
        "the post-extend scan must be a new cached allocation, not the prior universe"
    );
}

/// Clones copy the generation and share the DashMap. Extending the original
/// must not evict the clone's memoization for the universe it still describes.
#[test]
fn ts_fact_map_clone_keeps_cached_scans_after_original_extends() {
    use crate::codebase::dependencies::graph::TsFactLookup;
    use crate::codebase::ts_source::facts::{TsFactMap, TsFileFacts};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let mut facts = TsFactMap::new();
    let cloned = facts.clone();
    let calls = AtomicUsize::new(0);
    let compute = || -> anyhow::Result<_> {
        calls.fetch_add(1, Ordering::SeqCst);
        Ok(Default::default())
    };
    let cached = facts
        .get_or_compute_route_reachable_files(&cache_settings(), &compute)
        .unwrap();
    facts.extend(TsFactMap::from([(
        PathBuf::from("/repo/added.ts"),
        TsFileFacts::default(),
    )]));
    let from_clone = cloned
        .get_or_compute_route_reachable_files(&cache_settings(), &compute)
        .unwrap();
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "a clone that still has the pre-extend universe must keep the cached scan"
    );
    assert!(std::sync::Arc::ptr_eq(&cached, &from_clone));
    facts
        .get_or_compute_route_reachable_files(&cache_settings(), &compute)
        .unwrap();
    assert_eq!(
        calls.load(Ordering::SeqCst),
        2,
        "the extended original must miss the pre-extend generation"
    );
}

#[test]
fn ts_fact_map_bump_playwright_scan_generation_isolates_cache_keys() {
    use crate::codebase::dependencies::graph::TsFactLookup;
    use crate::codebase::ts_source::facts::TsFactMap;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let mut facts = TsFactMap::new();
    let calls = AtomicUsize::new(0);
    let compute = || -> anyhow::Result<_> {
        calls.fetch_add(1, Ordering::SeqCst);
        Ok(Default::default())
    };
    facts
        .get_or_compute_route_reachable_files(&cache_settings(), &compute)
        .unwrap();
    facts.bump_playwright_scan_generation();
    facts
        .get_or_compute_route_reachable_files(&cache_settings(), &compute)
        .unwrap();
    assert_eq!(
        calls.load(Ordering::SeqCst),
        2,
        "SharedTraversalContext invalidation bumps this generation so a rebuilt graph cannot reuse the prior universe's scan"
    );
}

/// Independently extended clones used to increment the same copied counter
/// and share DashMaps, so the second clone could reuse the first clone's
/// reachability for a different file universe.
#[test]
fn ts_fact_map_independent_clone_extends_use_unique_scan_generations() {
    use crate::codebase::dependencies::graph::TsFactLookup;
    use crate::codebase::ts_source::facts::{TsFactMap, TsFileFacts};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let mut first = TsFactMap::new();
    let mut second = first.clone();
    let calls = AtomicUsize::new(0);
    let compute = || -> anyhow::Result<_> {
        calls.fetch_add(1, Ordering::SeqCst);
        Ok(Default::default())
    };
    first.extend(TsFactMap::from([(
        PathBuf::from("/repo/a.ts"),
        TsFileFacts::default(),
    )]));
    second.extend(TsFactMap::from([(
        PathBuf::from("/repo/b.ts"),
        TsFileFacts::default(),
    )]));
    let from_first = first
        .get_or_compute_route_reachable_files(&cache_settings(), &compute)
        .unwrap();
    let from_second = second
        .get_or_compute_route_reachable_files(&cache_settings(), &compute)
        .unwrap();
    assert_eq!(
        calls.load(Ordering::SeqCst),
        2,
        "independently extended clones must not share a scan generation"
    );
    assert!(
        !std::sync::Arc::ptr_eq(&from_first, &from_second),
        "each extended clone must cache its own universe's reachability"
    );
}

/// insert/remove/get_mut change facts without going through `extend`.
#[test]
fn ts_fact_map_insert_remove_and_get_mut_invalidate_playwright_scan_caches() {
    use crate::codebase::dependencies::graph::TsFactLookup;
    use crate::codebase::ts_source::facts::{TsFactMap, TsFileFacts};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let mut facts = TsFactMap::new();
    let path = PathBuf::from("/repo/page.ts");
    let calls = AtomicUsize::new(0);
    let compute = || -> anyhow::Result<_> {
        calls.fetch_add(1, Ordering::SeqCst);
        Ok(Default::default())
    };
    facts
        .get_or_compute_route_reachable_files(&cache_settings(), &compute)
        .unwrap();
    facts.insert(path.clone(), TsFileFacts::default());
    facts
        .get_or_compute_route_reachable_files(&cache_settings(), &compute)
        .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 2, "insert must recompute");
    facts.get_mut(&path).expect("inserted path is present");
    facts
        .get_or_compute_route_reachable_files(&cache_settings(), &compute)
        .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 3, "get_mut must recompute");
    facts.remove(&path);
    facts
        .get_or_compute_route_reachable_files(&cache_settings(), &compute)
        .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 4, "remove must recompute");
}

#[test]
fn ts_fact_map_get_or_compute_methods_report_compute_errors() {
    use crate::codebase::dependencies::graph::TsFactLookup;
    use crate::playwright::selectors::AppSelector;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let facts = crate::codebase::ts_source::facts::TsFactMap::new();

    let selector_calls = AtomicUsize::new(0);
    let failing_selectors = || -> anyhow::Result<Vec<AppSelector>> {
        selector_calls.fetch_add(1, Ordering::SeqCst);
        anyhow::bail!("selector scan failed")
    };
    let error = facts
        .get_or_compute_app_selector_occurrences(&cache_settings(), false, &failing_selectors)
        .unwrap_err();
    assert!(error.to_string().contains("selector scan failed"));
    facts
        .get_or_compute_app_selector_occurrences(&cache_settings(), false, &failing_selectors)
        .unwrap_err();
    assert_eq!(selector_calls.load(Ordering::SeqCst), 1);

    let text_calls = AtomicUsize::new(0);
    let failing_text = || -> anyhow::Result<_> {
        text_calls.fetch_add(1, Ordering::SeqCst);
        anyhow::bail!("app text scan failed")
    };
    facts
        .get_or_compute_app_text_targets(&cache_settings(), &failing_text)
        .unwrap_err();
    facts
        .get_or_compute_app_text_targets(&cache_settings(), &failing_text)
        .unwrap_err();
    assert_eq!(text_calls.load(Ordering::SeqCst), 1);
}

#[test]
fn fallback_lookup_uses_primary_fetch_facts_before_parse_errors() {
    let path = PathBuf::from("/repo/file.ts");
    let primary = crate::codebase::check_facts::collect_check_facts_with_graph_files_and_playwright(
        Path::new("/repo"),
        vec![path.clone()],
        vec![path.clone()],
        crate::codebase::check_facts::CheckFactPlan::default(),
        None,
    );
    let fallback = crate::codebase::ts_source::facts::TsFactMap::from([(
        path.clone(),
        crate::codebase::ts_source::facts::TsFileFacts {
            parse_error: Some("e".into()),
            ..Default::default()
        },
    )]);
    let visible: crate::fx::PathSet = [path.clone()].into_iter().collect();
    let lookup = FallbackTsFactLookup::new(
        &primary,
        &fallback,
        false,
        std::slice::from_ref(&path),
        &visible,
    );
    assert!(lookup.get_playwright_fetch_facts(&path).is_some());
}
