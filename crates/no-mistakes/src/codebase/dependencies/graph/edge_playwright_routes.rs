/// Build Playwright route-test edges from the graph's canonical candidate
/// universe. The shared fact map supplies staged per-setting occurrences when
/// available; malformed individual tests remain non-fatal for graph builds.
#[path = "edge_playwright_route_layouts.rs"]
mod playwright_route_layouts;
use playwright_route_layouts::route_and_layout_edges;

/// Build Playwright route-test edges for every resolved frontend app.
/// `prepared_settings` empty means no app was prepared (e.g. an unprepared
/// ad hoc caller) — fall back to loading exactly one `Settings` from disk,
/// the pre-multi-app behavior. A prepared caller instead supplies one
/// `Settings` per app (see `PreparedGraphConfig`), so a multi-`type:
/// nextjs`-app repository's route edges aren't limited to a single
/// arbitrarily-chosen app.
fn collect_playwright_route_edges_from_snapshot(
    root: &Path,
    config_path: Option<&Path>,
    all_files: &[PathBuf],
    facts: Option<&dyn TsFactLookup>,
    snapshot: &crate::playwright::fsutil::VisiblePathSnapshot,
    prepared_settings: &[crate::playwright::config::Settings],
    interner: &PathInterner,
) -> Vec<Edge> {
    if !prepared_settings.is_empty() {
        let mut edges: Vec<Edge> = prepared_settings
            .iter()
            .flat_map(|settings| {
                collect_playwright_route_edges_for_settings(
                    root, all_files, facts, snapshot, settings, interner,
                )
            })
            .collect();
        edges.sort();
        edges.dedup();
        return edges;
    }
    let Ok(settings) = crate::playwright::config::load_settings_from_visible(
        root,
        config_path,
        &[],
        None,
        None,
        snapshot,
    ) else {
        return Vec::new();
    };
    collect_playwright_route_edges_for_settings(
        root, all_files, facts, snapshot, &settings, interner,
    )
}

fn collect_playwright_route_edges_for_settings(
    root: &Path,
    all_files: &[PathBuf],
    facts: Option<&dyn TsFactLookup>,
    snapshot: &crate::playwright::fsutil::VisiblePathSnapshot,
    settings: &crate::playwright::config::Settings,
    interner: &PathInterner,
) -> Vec<Edge> {
    let all_file_set: HashSet<PathBuf> = all_files.iter().cloned().collect();
    let frontend_root = root.join(&settings.frontend_root);
    let compute_routes = || {
        let route_paths = snapshot.paths_for(&frontend_root);
        let mut routes = crate::routes::collect_routes_from_visible_paths(
            &frontend_root,
            &route_paths,
            &["page"],
        );
        let virtual_routes = crate::routes::rewrites::expand_rewrites(&settings.rewrites, &routes);
        routes.extend(virtual_routes);
        routes
    };
    let routes = match facts {
        Some(facts) => facts.get_or_compute_playwright_routes(settings, &compute_routes),
        None => Arc::new(compute_routes()),
    };
    if routes.is_empty() {
        return Vec::new();
    }
    let test_files = match facts
        .and_then(|facts| facts.get_playwright_test_files(settings.project.as_deref()))
    {
        Some(test_files) => test_files,
        None => {
            let Ok(playwright) = crate::playwright::playwright_config::load_many(
                root,
                &settings.playwright_configs,
                settings.project.as_deref(),
            ) else {
                return Vec::new();
            };
            let Ok(test_files) =
                crate::playwright::analysis::discover::discover_test_files_from_visible(
                    root,
                    settings,
                    &playwright,
                    snapshot,
                )
            else {
                return Vec::new();
            };
            Arc::new(test_files)
        }
    };
    let route_idx = crate::playwright::analysis::routes_index::route_index(root, &routes);
    let selector_regexes = crate::playwright::selectors::compile_selector_regexes_with_html_ids(
        &settings.selector_attributes,
        &settings.component_selector_attributes,
        settings.html_ids,
    );
    let selector_index = Default::default();
    let test_analysis = crate::playwright::analysis::context::TestAnalysisContext {
        root,
        route_index: &route_idx,
        selector_index: &selector_index,
        navigation_helpers: &settings.navigation_helpers,
        selector_wrappers: &settings.selector_wrappers,
        selector_regexes: &selector_regexes,
        test_policy: crate::playwright::playwright_tests::TestPolicy {
            assert_conditional_tests: false,
            allow_skipped_tests: false,
        },
    };

    let test_edges: Vec<crate::playwright::analysis::types::Edge> = test_files
        .par_iter()
        .filter_map(|test_file| {
            match facts.and_then(|facts| facts.get_playwright_facts(&test_file.path)) {
                Some(playwright) => {
                    let attributes = test_file.test_id_attributes();
                    let key = crate::codebase::check_facts::PlaywrightOccurrenceKey::new(
                        &settings.navigation_helpers,
                        &settings.selector_wrappers,
                        &settings.selector_attributes,
                        &settings.component_selector_attributes,
                        settings.html_ids,
                        &attributes,
                    );
                    playwright.select(&key).map(|occurrences| {
                        crate::playwright::analysis::test_file::analyze_test_occurrences(
                            test_file,
                            &test_analysis,
                            &occurrences,
                        )
                        .edges
                    })
                }
                None => {
                    if facts.is_some_and(|facts| {
                        facts.get_playwright_parse_error(&test_file.path).is_some()
                    }) {
                        return None;
                    }
                    crate::playwright::analysis::test_file::analyze_test_file(
                        test_file,
                        &test_analysis,
                    )
                    .ok()
                    .map(|analysis| analysis.edges)
                }
            }
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
        edges.extend(route_and_layout_edges(
            test_file,
            page_file,
            &frontend_root,
            &all_file_set,
            interner,
        ));
    }
    edges.sort();
    edges.dedup();
    edges
}
