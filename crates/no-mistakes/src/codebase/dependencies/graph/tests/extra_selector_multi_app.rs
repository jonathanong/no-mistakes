// ── Multi-frontend-app selector edge collection ──────────────────────────

/// Regression test (review finding on the multi-frontend-app change): one
/// prepared app's selector analysis failing must not discard edges already
/// found for another, successfully-analyzed app — `edge_playwright_routes`
/// already tolerates a per-app failure this way, and `edge_playwright_selectors`
/// must match it instead of propagating the first error via `?` and losing
/// every other app's edges.
#[test]
fn collect_playwright_selector_edges_skips_a_failing_app_but_keeps_the_others() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/nextjs-selectors/selector-covered/fixture"),
    );
    let all_files = crate::codebase::ts_source::discover_files(&root, &[]);
    let snapshot = crate::playwright::fsutil::VisiblePathSnapshot::new(&root);

    let good_settings =
        crate::playwright::config::test_support::load_settings(&root, None, &[], None).unwrap();
    let mut broken_settings = good_settings.clone();
    broken_settings.playwright_configs = vec![root.join("does-not-exist.playwright.config.ts")];

    let prepared_settings = vec![broken_settings, good_settings];
    let edges = collect_playwright_selector_edges_with_graph(
        &root,
        None,
        PlaywrightSelectorEdgeInputs {
            all_files: &all_files,
            facts: None,
            partial_graph: None,
            graph_tsconfig: None,
            snapshot: &snapshot,
            prepared_settings: &prepared_settings,
        },
    )
    .unwrap();

    assert!(
        !edges.is_empty(),
        "expected the working app's selector edges to survive the broken app's failure"
    );
}

/// Source guard: prepared-app selector analysis must stay on `par_iter` and
/// flatten-then-sort/dedup. A serial `for settings in` loop would still
/// produce the same edges after sort/dedup, so a behavioral fixture cannot
/// catch the regression this change is meant to lock in.
#[test]
fn collect_playwright_selector_edges_analyzes_prepared_apps_in_parallel() {
    let source = include_str!("../edge_playwright_selectors.rs");
    let body = graph_files_source_function_body(
        source,
        "pub(super) fn collect_playwright_selector_edges_with_graph(",
    );
    assert!(
        body.contains("par_iter"),
        "prepared apps are independent after the base graph exists and must be scanned in parallel"
    );
    assert!(
        !body.contains("for settings in"),
        "serial per-app for-loop must not return; flatten then sort/dedup"
    );
    assert!(
        body.contains("flat_map")
            && body.contains("edges.sort()")
            && body.contains("edges.dedup()"),
        "parallel app results must flatten, then sort and dedup for byte-identical output"
    );
    assert!(
        body.contains("with_owned_request_parse_cache")
            && !body.contains("with_request_parse_cache("),
        "Rayon app tasks must install an owned parse cache, not inherit a sibling request's"
    );
    assert!(
        body.contains("TimingKind::Parallel"),
        "overlapping per-app selector timings must be marked non-additive"
    );
}
