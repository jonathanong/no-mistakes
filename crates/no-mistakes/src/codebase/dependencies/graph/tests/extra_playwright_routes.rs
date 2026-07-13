#[test]
fn playwright_layout_edges_use_discovered_file_set() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("playwright-impact-routing"));
    let frontend_root = root.join("web/app");
    let page = root.join("web/app/users/[id]/page.tsx");
    let all_files: HashSet<PathBuf> = [
        root.join("web/app/layout.tsx"),
        root.join("web/app/users/layout.tsx"),
        root.join("web/app/users/[id]/page.tsx"),
    ]
    .into();

    assert_eq!(
        collect_layout_chain_files_from_file_set(&page, &frontend_root, &all_files),
        vec![
            root.join("web/app/users/layout.tsx"),
            root.join("web/app/layout.tsx")
        ]
    );
}

#[test]
fn playwright_route_edges_use_app_root_and_filter_graph_files() {
    let root =
        crate::codebase::ts_resolver::normalize_path(&fixture("playwright-route-edges-v2"));
    let test_file = root.join("tests/e2e/home.spec.ts");
    let invalid_test_file = root.join("tests/e2e/invalid.spec.ts");
    let page = root.join("web/app/page.tsx");
    let layout = root.join("web/app/layout.tsx");
    let bad_app_file = root.join("web/app/bad.tsx");
    let config = root.join("playwright.config.mts");

    let edges = collect_playwright_route_edges(
        &root,
        None,
        &[
            test_file.clone(),
            invalid_test_file,
            page.clone(),
            layout.clone(),
            bad_app_file,
            config,
            root.join(".no-mistakes.yml"),
        ],
        None,
    );
    assert!(
        edges.contains(&(
            NodeId::File(test_file.clone()),
            NodeId::File(page.clone()),
            EdgeKind::RouteTest
        )),
        "expected route edge, got {edges:?}"
    );
    assert!(edges.contains(&(NodeId::File(page.clone()), NodeId::File(layout), EdgeKind::Layout)));

    let filtered_edges = collect_playwright_route_edges(
        &root,
        None,
        &[test_file, root.join(".no-mistakes.yml")],
        None,
    );
    assert!(
        !filtered_edges.iter().any(|(_, target, kind)| {
            target == &NodeId::File(page.clone()) && *kind == EdgeKind::RouteTest
        }),
        "route edges should not introduce files outside the graph file set"
    );
}

/// Regression test for the graph-edge/rule-pipeline route-scan duplication fixed in this
/// change: `collect_playwright_route_edges` must resolve routes via the shared
/// `facts.get_or_compute_playwright_routes` cache when a caller has one, not by
/// independently re-collecting routes from disk. Asserts on a disagreement, not output
/// equality (`crates/CLAUDE.md`: "assert on a call count, not value equality" / "construct a
/// case where the two approaches would disagree") — pre-populates the shared cache with an
/// empty route list, deliberately different from the real `web/app/page.tsx` route this
/// fixture has on disk. A version that bypasses the cache would still find that real route and
/// produce a `RouteTest` edge for it; a version that correctly shares the cache must see the
/// pre-populated empty list and produce no edges at all.
#[test]
fn playwright_route_edges_reuse_shared_route_cache_instead_of_rescanning() {
    use crate::codebase::check_facts::CheckFactMap;
    use crate::codebase::dependencies::graph::TsFactLookup;

    let root = crate::codebase::ts_resolver::normalize_path(&fixture("playwright-route-edges-v2"));
    let test_file = root.join("tests/e2e/home.spec.ts");
    let page = root.join("web/app/page.tsx");
    let layout = root.join("web/app/layout.tsx");
    let all_files = [
        test_file,
        page,
        layout,
        root.join("playwright.config.mts"),
        root.join(".no-mistakes.yml"),
    ];

    let facts = CheckFactMap::default();
    // Pre-populate the shared route cache with an empty list before the producer ever runs,
    // so a correct implementation sees this stale-but-cached value instead of rescanning disk.
    let prepopulated = facts.get_or_compute_playwright_routes(&Vec::new);
    assert!(prepopulated.is_empty(), "sanity check on the pre-populated cache value");

    let edges = collect_playwright_route_edges(&root, None, &all_files, Some(&facts));

    assert!(
        edges.is_empty(),
        "expected no edges: the producer must have used the pre-populated (empty) shared \
         route cache rather than independently rediscovering the real page.tsx route on disk, \
         got {edges:?}"
    );
}

/// Regression test (Codex review finding): `collect_playwright_route_edges` must load
/// Playwright settings from the same `config_path` its caller resolves, not a hardcoded `None`
/// — `get_or_compute_playwright_routes` is an unkeyed cache, so if this producer's settings
/// silently diverged from the `playwright` rule's (which does thread its resolved config path,
/// see `playwright/rules.rs`), the shared route list would reflect whichever caller happened to
/// populate the cache first for the *other* caller's config too. Constructs a disagreement
/// case: same fixture and `all_files`, only `config_path` differs — `None` (discovered config)
/// must succeed and produce real edges, while an explicit, nonexistent `config_path` must fail
/// `load_settings` and produce no edges. If `config_path` were still hardcoded to `None`
/// internally, both calls would produce identical (non-empty) results.
#[test]
fn playwright_route_edges_load_settings_from_the_supplied_config_path() {
    let root =
        crate::codebase::ts_resolver::normalize_path(&fixture("playwright-route-edges-v2"));
    let test_file = root.join("tests/e2e/home.spec.ts");
    let page = root.join("web/app/page.tsx");
    let layout = root.join("web/app/layout.tsx");
    let all_files = [
        test_file,
        page,
        layout,
        root.join("playwright.config.mts"),
        root.join(".no-mistakes.yml"),
    ];

    let discovered = collect_playwright_route_edges(&root, None, &all_files, None);
    assert!(
        !discovered.is_empty(),
        "sanity check: the discovered (None) config path must produce real edges"
    );

    let missing_config_path = root.join("does-not-exist.no-mistakes.yml");
    let explicit_missing =
        collect_playwright_route_edges(&root, Some(&missing_config_path), &all_files, None);
    assert!(
        explicit_missing.is_empty(),
        "an explicit config_path must reach load_settings (and fail, since this path doesn't \
         exist) rather than being silently ignored in favor of the discovered config, \
         got {explicit_missing:?}"
    );
}

#[test]
fn playwright_route_edges_match_unresolved_interpolations_to_dynamic_segment() {
    // #391/#397: every navigation form whose final segment is an unresolved interpolation must
    // edge to the dynamic `[idOrUsername]` page, but never to the sibling literal
    // `/user/settings` page. Each form lives in its own spec file so a single form's failure is
    // caught — a combined spec would let one passing navigation mask the others. The `let`
    // cases (including the reassigned-in-`beforeAll` shape from #397) are the regression: an
    // unresolved `let` initialized to "" must not collapse the path to a non-matching `/user/`.
    let root =
        crate::codebase::ts_resolver::normalize_path(&fixture("playwright-interpolated-routes"));
    let dynamic_page = root.join("web/app/(user)/user/[idOrUsername]/page.tsx");
    let literal_page = root.join("web/app/user/settings/page.tsx");
    let spec_files = [
        "tests/e2e/let-template.spec.ts",
        "tests/e2e/let-concat.spec.ts",
        "tests/e2e/let-reassigned.spec.ts",
        "tests/e2e/const-goto.spec.ts",
    ]
    .map(|spec| root.join(spec));

    let mut all_files = vec![
        dynamic_page.clone(),
        literal_page.clone(),
        root.join("playwright.config.mts"),
        root.join(".no-mistakes.yml"),
    ];
    all_files.extend(spec_files.iter().cloned());

    let edges = collect_playwright_route_edges(&root, None, &all_files, None);

    let dynamic_node = NodeId::File(dynamic_page);
    let literal_node = NodeId::File(literal_page);
    for spec in &spec_files {
        let spec_node = NodeId::File(spec.clone());
        assert!(
            edges.contains(&(spec_node.clone(), dynamic_node.clone(), EdgeKind::RouteTest)),
            "expected route edge from {} to the dynamic page, got {edges:?}",
            spec.display()
        );
        assert!(
            !edges.contains(&(spec_node, literal_node.clone(), EdgeKind::RouteTest)),
            "interpolated navigation in {} must not select the sibling literal route, got {edges:?}",
            spec.display()
        );
    }
}

#[test]
fn playwright_route_edges_cover_defensive_config_errors() {
    for name in [
        "playwright-route-edges-invalid-settings",
        "playwright-route-edges-invalid-config",
        "playwright-route-edges-invalid-test-glob",
    ] {
        let root = crate::codebase::ts_resolver::normalize_path(&fixture(name));
        assert!(
            collect_playwright_route_edges(&root, None, &[root.join("web/app/page.tsx")], None)
                .is_empty(),
            "{name} should not produce route edges"
        );
    }
}
