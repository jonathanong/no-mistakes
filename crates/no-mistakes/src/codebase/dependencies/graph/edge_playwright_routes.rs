/// Reuses the shared `TsFactLookup` (when the caller has one) for both the
/// app-wide route scan and the per-test-file Playwright analysis, instead of
/// independently re-collecting routes and re-parsing every test file
/// sequentially — the `playwright` rule's own check pipeline
/// (`playwright::analysis::pipeline_setup`/`pipeline_test_analysis`) already
/// does both of these memoized and in parallel via the same `facts`. See
/// `crates/CLAUDE.md`'s "Duplicate full-repo work across independent call
/// paths".
fn collect_playwright_route_edges(
    root: &Path,
    all_files: &[PathBuf],
    facts: Option<&dyn TsFactLookup>,
) -> Vec<Edge> {
    let all_file_set: HashSet<PathBuf> = all_files.iter().cloned().collect();
    let Ok(settings) = crate::playwright::config::load_settings(root, None, &[], None) else {
        return Vec::new();
    };
    let frontend_root = root.join(&settings.frontend_root);
    let compute_routes = || {
        let mut routes = crate::routes::collect_routes(&frontend_root, &["page"]);
        let virtual_routes =
            crate::routes::rewrites::expand_rewrites(&settings.rewrites, &routes);
        routes.extend(virtual_routes);
        routes
    };
    let routes = match facts {
        Some(facts) => facts.get_or_compute_playwright_routes(&compute_routes),
        None => std::sync::Arc::new(compute_routes()),
    };
    if routes.is_empty() {
        return Vec::new();
    }
    let Ok(playwright) = crate::playwright::playwright_config::load_many(
        root,
        &settings.playwright_configs,
        settings.project.as_deref(),
    ) else {
        return Vec::new();
    };
    let Ok(test_files) =
        crate::playwright::analysis::discover::discover_test_files(root, &settings, &playwright)
    else {
        return Vec::new();
    };
    let route_idx = crate::playwright::analysis::routes_index::route_index(root, &routes);
    let selector_regexes = crate::playwright::selectors::compile_selector_regexes_with_html_ids(
        &settings.selector_attributes,
        &settings.component_selector_attributes,
        settings.html_ids,
    );
    let empty_selector_targets = Vec::new();
    let selector_index = Default::default();
    let app_text_targets = Vec::new();
    let route_reachable_files = Default::default();
    let test_analysis = crate::playwright::analysis::context::TestAnalysisContext {
        root,
        route_index: &route_idx,
        app_selector_targets: &empty_selector_targets,
        selector_index: &selector_index,
        app_text_targets: &app_text_targets,
        route_reachable_files: &route_reachable_files,
        navigation_helpers: &settings.navigation_helpers,
        selector_regexes: &selector_regexes,
        test_policy: crate::playwright::playwright_tests::TestPolicy {
            assert_conditional_tests: false,
            allow_skipped_tests: false,
        },
    };

    // Deliberately does not reuse `pipeline_test_analysis::analyze_test_files`
    // as-is: that helper propagates the first per-file analysis error via `?`,
    // which is correct for the `playwright` rule check (see `pipeline.rs`,
    // same `?`) but wrong for a graph edge producer — one malformed test file
    // must not abort the whole `DepGraph` build that every other check depends
    // on. Mirror its per-file dispatch (shared facts when available, parsing
    // otherwise) but skip a failing file instead of aborting the batch, same
    // as this function's pre-existing behavior.
    let test_edges: Vec<crate::playwright::analysis::types::Edge> = test_files
        .par_iter()
        .filter_map(|test_file| {
            let file_analysis = match facts.and_then(|facts| facts.get_playwright_facts(&test_file.path)) {
                Some(playwright) => crate::playwright::analysis::test_file::analyze_test_occurrences(
                    test_file,
                    &test_analysis,
                    playwright.urls.clone(),
                    playwright.selectors.clone(),
                    playwright.text_locators.clone(),
                    playwright.helper_references.clone(),
                ),
                None => crate::playwright::analysis::test_file::analyze_test_file(test_file, &test_analysis)
                    .ok()?,
            };
            Some(file_analysis.edges)
        })
        .flatten()
        .collect();

    let mut edges = Vec::new();
    for edge in test_edges {
        let crate::playwright::analysis::types::Edge::Route {
            test_file,
            route_file,
            ..
        } = edge
        else {
            continue;
        };
        let test_file = root.join(test_file.as_str());
        let page_file = root.join(route_file.as_str());
        if !all_file_set.contains(&test_file) || !all_file_set.contains(&page_file) {
            continue;
        }
        edges.push((
            NodeId::File(test_file),
            NodeId::File(page_file.clone()),
            EdgeKind::RouteTest,
        ));
        for layout_file in
            collect_layout_chain_files_from_file_set(&page_file, &frontend_root, &all_file_set)
        {
            edges.push((
                NodeId::File(page_file.clone()),
                NodeId::File(layout_file),
                EdgeKind::Layout,
            ));
        }
    }
    edges.sort();
    edges.dedup();
    edges
}

fn collect_layout_chain_files_from_file_set(
    route_file: &Path,
    frontend_root: &Path,
    all_files: &HashSet<PathBuf>,
) -> Vec<PathBuf> {
    let mut layout_files = Vec::new();
    let mut current = route_file.parent();
    while let Some(parent) = current {
        if !parent.starts_with(frontend_root) {
            break;
        }

        for stem in ["layout", "loading", "error", "not-found", "template"] {
            for ext in ["tsx", "ts", "jsx", "js"] {
                let layout_file = parent.join(format!("{stem}.{ext}"));
                if all_files.contains(&layout_file) {
                    layout_files.push(layout_file);
                }
            }
        }

        current = parent.parent();
    }

    layout_files
}
