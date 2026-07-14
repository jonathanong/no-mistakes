/// Build Playwright route-test edges from the graph's canonical candidate
/// universe. The shared fact map supplies staged per-setting occurrences when
/// available; malformed individual tests remain non-fatal for graph builds.
#[path = "edge_playwright_route_layouts.rs"]
mod playwright_route_layouts;
use playwright_route_layouts::route_and_layout_edges;

fn collect_playwright_route_edges_from_snapshot(
    root: &Path,
    config_path: Option<&Path>,
    all_files: &[PathBuf],
    facts: Option<&dyn TsFactLookup>,
    snapshot: &crate::playwright::fsutil::VisiblePathSnapshot,
    prepared_settings: Option<&crate::playwright::config::Settings>,
) -> Vec<Edge> {
    let all_file_set: HashSet<PathBuf> = all_files.iter().cloned().collect();
    let loaded_settings;
    let settings = if let Some(settings) = prepared_settings {
        settings
    } else {
        let Ok(settings) = crate::playwright::config::load_settings_from_visible(
            root,
            config_path,
            &[],
            None,
            snapshot,
        ) else {
            return Vec::new();
        };
        loaded_settings = settings;
        &loaded_settings
    };
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
        ));
    }
    edges.sort();
    edges.dedup();
    edges
}
