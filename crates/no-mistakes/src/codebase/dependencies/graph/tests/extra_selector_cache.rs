// ── shared app-selector-occurrences cache (CheckFactMap) ─────────────────

/// Regression test: `CheckFactMap::get_or_compute_app_selector_occurrences`
/// must call `compute` at most once per distinct `scan_html_ids` key — this
/// is what actually makes `no-mistakes check` dedupe the app-wide selector
/// scan across `playwright::rules::check_with_facts` and
/// `forbidden_dependencies`'s `DepGraph` build (previously each paid the
/// full scan independently). Asserting on the *returned value* alone
/// wouldn't prove this — a non-caching implementation returns the same
/// value too, just by recomputing it; asserting on the call count does.
#[test]
fn get_or_compute_app_selector_occurrences_caches_per_scan_html_ids_key() {
    use crate::codebase::check_facts::CheckFactMap;
    use crate::codebase::dependencies::graph::TsFactLookup;
    use crate::playwright::selectors::AppSelector;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let facts = CheckFactMap::default();
    let calls = AtomicUsize::new(0);
    let compute = || -> anyhow::Result<Vec<AppSelector>> {
        calls.fetch_add(1, Ordering::SeqCst);
        Ok(Vec::new())
    };

    let first = facts
        .get_or_compute_app_selector_occurrences(&cache_settings(), false, &compute)
        .unwrap();
    let second = facts
        .get_or_compute_app_selector_occurrences(&cache_settings(), false, &compute)
        .unwrap();
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "a second call with the same scan_html_ids key must reuse the cached result, not recompute"
    );
    assert!(
        std::sync::Arc::ptr_eq(&first, &second),
        "cached calls must return the same Arc allocation, not merely an equal value"
    );

    facts
        .get_or_compute_app_selector_occurrences(&cache_settings(), true, &compute)
        .unwrap();
    assert_eq!(
        calls.load(Ordering::SeqCst),
        2,
        "a different scan_html_ids key is a real input to the scan (see doc comment) and must recompute"
    );
}

/// Regression test: a failing `compute` must still be cached (as a `String`,
/// since `anyhow::Error` isn't `Clone`) and reported back through `Result`,
/// not just the success path — and a second call with a failing `compute`
/// must reuse the cached error rather than recomputing (same call-count
/// discipline as the success-path tests above).
#[test]
fn get_or_compute_methods_cache_and_report_compute_errors() {
    use crate::codebase::check_facts::CheckFactMap;
    use crate::codebase::dependencies::graph::TsFactLookup;
    use crate::playwright::selectors::AppSelector;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let facts = CheckFactMap::default();

    let selector_calls = AtomicUsize::new(0);
    let failing_selectors = || -> anyhow::Result<Vec<AppSelector>> {
        selector_calls.fetch_add(1, Ordering::SeqCst);
        anyhow::bail!("selector scan failed")
    };
    let first_error = facts
        .get_or_compute_app_selector_occurrences(&cache_settings(), false, &failing_selectors)
        .unwrap_err();
    assert!(first_error.to_string().contains("selector scan failed"));
    let second_error = facts
        .get_or_compute_app_selector_occurrences(&cache_settings(), false, &failing_selectors)
        .unwrap_err();
    assert!(second_error.to_string().contains("selector scan failed"));
    assert_eq!(
        selector_calls.load(Ordering::SeqCst),
        1,
        "a cached error must not trigger a recompute"
    );

    let text_target_calls = AtomicUsize::new(0);
    let failing_text_targets = || -> anyhow::Result<_> {
        text_target_calls.fetch_add(1, Ordering::SeqCst);
        anyhow::bail!("app text scan failed")
    };
    facts
        .get_or_compute_app_text_targets(&cache_settings(), &failing_text_targets)
        .unwrap_err();
    facts
        .get_or_compute_app_text_targets(&cache_settings(), &failing_text_targets)
        .unwrap_err();
    assert_eq!(text_target_calls.load(Ordering::SeqCst), 1);

    let route_reachable_calls = AtomicUsize::new(0);
    let failing_route_reachable = || -> anyhow::Result<_> {
        route_reachable_calls.fetch_add(1, Ordering::SeqCst);
        anyhow::bail!("route reachability scan failed")
    };
    facts
        .get_or_compute_route_reachable_files(&cache_settings(), &failing_route_reachable)
        .unwrap_err();
    facts
        .get_or_compute_route_reachable_files(&cache_settings(), &failing_route_reachable)
        .unwrap_err();
    assert_eq!(route_reachable_calls.load(Ordering::SeqCst), 1);
}

/// Regression test: `get_or_compute_route_reachable_files` — the cache behind
/// this session's largest measured win (~8s per call on a real monorepo,
/// dropping to ~0 on the second call) — must call `compute` at most once,
/// with no key needed (unlike `app_selector_occurrences`, this scan has no
/// caller-varying input; see the trait doc comment).
#[test]
fn get_or_compute_route_reachable_files_caches_across_calls() {
    use crate::codebase::check_facts::CheckFactMap;
    use crate::codebase::dependencies::graph::TsFactLookup;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let facts = CheckFactMap::default();
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
        "a second call must reuse the cached result, not recompute the reachability scan"
    );
    assert!(
        std::sync::Arc::ptr_eq(&first, &second),
        "cached calls must return the same Arc allocation, not merely an equal value"
    );
}

/// Regression test: `get_or_compute_playwright_routes` and
/// `get_or_compute_app_text_targets` — the two smaller keyless caches added
/// alongside `route_reachable_files` — must each call `compute` at most once.
#[test]
fn get_or_compute_routes_and_app_text_targets_cache_across_calls() {
    use crate::codebase::check_facts::CheckFactMap;
    use crate::codebase::dependencies::graph::TsFactLookup;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let facts = CheckFactMap::default();

    let route_calls = AtomicUsize::new(0);
    let compute_routes = || -> Vec<crate::routes::Route> {
        route_calls.fetch_add(1, Ordering::SeqCst);
        Vec::new()
    };
    facts.get_or_compute_playwright_routes(&cache_settings(), &compute_routes);
    facts.get_or_compute_playwright_routes(&cache_settings(), &compute_routes);
    assert_eq!(
        route_calls.load(Ordering::SeqCst),
        1,
        "a second call must reuse the cached routes, not recompute"
    );

    let text_target_calls = AtomicUsize::new(0);
    let compute_text_targets = || -> anyhow::Result<_> {
        text_target_calls.fetch_add(1, Ordering::SeqCst);
        Ok(Vec::new())
    };
    facts
        .get_or_compute_app_text_targets(&cache_settings(), &compute_text_targets)
        .unwrap();
    facts
        .get_or_compute_app_text_targets(&cache_settings(), &compute_text_targets)
        .unwrap();
    assert_eq!(
        text_target_calls.load(Ordering::SeqCst),
        1,
        "a second call must reuse the cached app text targets, not recompute"
    );
}

/// `TsFactMap` never overrides the `get_or_compute_*` cache methods (only
/// `CheckFactMap` does — it's the only implementor that ever needs to share
/// these scans across call sites), so it exercises the trait's default
/// "always call compute, no caching" bodies. Every `compute` still runs
/// exactly once per call here (there's nothing to cache), which is the
/// correct, expected behavior for this fallback path.
#[test]
fn ts_fact_map_uses_uncached_trait_defaults_for_get_or_compute_methods() {
    use crate::codebase::dependencies::graph::TsFactLookup;
    use crate::codebase::ts_source::facts::TsFactMap;

    let facts = TsFactMap::new();

    let selectors = facts
        .get_or_compute_app_selector_occurrences(&cache_settings(), false, &|| Ok(Vec::new()))
        .unwrap();
    assert!(selectors.is_empty());

    let routes = facts.get_or_compute_playwright_routes(&cache_settings(), &|| {
        Vec::<crate::routes::Route>::new()
    });
    assert!(routes.is_empty());

    let app_text_targets = facts
        .get_or_compute_app_text_targets(&cache_settings(), &|| Ok(Vec::new()))
        .unwrap();
    assert!(app_text_targets.is_empty());

    let route_reachable_files = facts
        .get_or_compute_route_reachable_files(&cache_settings(), &|| Ok(Default::default()))
        .unwrap();
    assert!(route_reachable_files.is_empty());
}

#[test]
fn graph_build_plan_playwright_selectors_enabled_in_all() {
    let plan = GraphBuildPlan::all();
    assert!(plan.playwright_selectors);
}

#[test]
fn graph_build_plan_playwright_selectors_from_allowed() {
    let allowed: HashSet<EdgeKind> = [EdgeKind::Selector].into();
    let plan = GraphBuildPlan::from_allowed(Some(&allowed));
    assert!(plan.playwright_selectors);
    assert!(!plan.playwright_routes);
    assert!(!plan.imports);
}

#[test]
fn graph_build_plan_playwright_selectors_not_set_by_default() {
    let plan = GraphBuildPlan::default();
    assert!(!plan.playwright_selectors);
}
