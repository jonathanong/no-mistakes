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
