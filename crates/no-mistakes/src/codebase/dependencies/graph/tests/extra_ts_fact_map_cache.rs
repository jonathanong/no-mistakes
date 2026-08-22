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
    assert!(second.to_string().contains("route reachability scan failed"));
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
