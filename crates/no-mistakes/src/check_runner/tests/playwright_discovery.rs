use super::run_all;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn check_without_playwright_rules_discovers_once() {
    // Equal findings still pass if Playwright walks the repo again. The
    // session records that single discovery on these counters.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check-runner/empty");
    let observer = no_mistakes::diagnostics::InvocationObserver::new(true);
    let results = {
        let _guard = no_mistakes::diagnostics::InvocationGuard::install(Arc::clone(&observer));
        run_all(root, None, None).unwrap()
    };

    assert!(results.rules.is_empty(), "{:?}", results.rules);
    let work = observer.snapshot().work;
    assert_eq!(
        work.get("discovery.requests").copied(),
        Some(1),
        "{work:#?}"
    );
    assert_eq!(
        work.get("discovery.cache_hits").copied().unwrap_or(0),
        0,
        "{work:#?}"
    );
    assert_eq!(work.get("discovery.roots").copied(), Some(1), "{work:#?}");
}

#[test]
fn empty_playwright_plan_returns_before_building_a_snapshot() {
    let source = include_str!("../../playwright/rules/fact_plan.rs");
    let early = source
        .find("return Ok(None)")
        .expect("empty playwright plan returns before discovery");
    let walk = source
        .find("VisiblePathSnapshot::new")
        .expect("standalone callers still build a snapshot after the empty return");
    assert!(
        early < walk,
        "empty playwright config must return before VisiblePathSnapshot::new"
    );
}

#[test]
fn playwright_fact_plan_reuses_the_session_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/playwright-route-edges-v2/fixture");
    let observer = no_mistakes::diagnostics::InvocationObserver::new(true);
    let _guard = no_mistakes::diagnostics::InvocationGuard::install(Arc::clone(&observer));
    let session =
        no_mistakes::codebase::analysis_session::AnalysisSession::new(Some(Arc::clone(&observer)));
    let snapshot = session.visible_paths(&root);
    let plan = no_mistakes::playwright::rules::fact_plan_for_consumers(
        &root,
        None,
        &no_mistakes::config::v2::NoMistakesConfig::default(),
        no_mistakes::playwright::rules::PlaywrightFactConsumers {
            graph_selectors: true,
            graph_routes: true,
        },
        Some(snapshot.as_ref()),
    )
    .unwrap();

    assert!(plan.is_some());
    let work = observer.snapshot().work;
    assert_eq!(
        work.get("discovery.requests").copied(),
        Some(1),
        "{work:#?}"
    );
    assert_eq!(
        work.get("discovery.cache_hits").copied().unwrap_or(0),
        0,
        "{work:#?}"
    );
    assert_eq!(work.get("discovery.roots").copied(), Some(1), "{work:#?}");
}
