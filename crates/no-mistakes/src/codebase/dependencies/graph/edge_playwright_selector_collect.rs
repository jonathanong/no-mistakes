/// Per-app selector/locator-text edges from prepared Playwright facts.
/// Graph build must not call the standalone `analyze_with_policy` pipeline.
fn collect_playwright_selector_edges_for_settings(
    root: &Path,
    settings: &crate::playwright::config::Settings,
    inputs: &PlaywrightSelectorEdgeInputs<'_>,
) -> Result<Vec<Edge>> {
    let test_policy = crate::playwright::playwright_tests::TestPolicy {
        assert_conditional_tests: false,
        allow_skipped_tests: false,
    };
    let unique_policy = crate::playwright::analysis::types::UniqueSelectorPolicy::default();
    let selector_regexes = crate::playwright::selectors::compile_selector_regexes_with_html_ids(
        &settings.selector_attributes,
        &settings.component_selector_attributes,
        settings.html_ids,
    );
    let test_files = crate::playwright::analysis::pipeline_setup::discover_playwright_test_files(
        root,
        settings,
        inputs.facts,
        inputs.snapshot,
    )?;
    let app_setup = crate::playwright::analysis::pipeline_setup::collect_app_selectors(
        root,
        settings,
        &unique_policy,
        inputs.facts,
        inputs.snapshot,
        Some(&selector_regexes),
    )?;
    let wrapper_resolution =
        selector_wrapper_resolution(root, settings, inputs.snapshot, inputs.route_import())?;
    let sources = inputs.snapshot.source_store_for(root);
    let (prepared, demand) = crate::playwright::analysis::pipeline_occurrences::prepare_test_files(
        test_files,
        settings,
        &selector_regexes,
        crate::playwright::analysis::pipeline_occurrences::PrepareTestFilesOptions {
            test_policy,
            skip_test_file_errors: true,
            facts: inputs.facts,
            selection:
                crate::playwright::analysis::pipeline_occurrences::CachedOccurrenceSelection::Exact,
            module_resolution: wrapper_resolution.as_ref(),
            sources: Some(sources.as_ref()),
        },
    )?;
    finish_selector_edges_for_settings(
        root,
        settings,
        inputs,
        SelectorEdgeFinishInputs {
            test_policy,
            selector_regexes: &selector_regexes,
            app_selectors: &app_setup.app_selectors,
            prepared,
            demand,
        },
    )
}

struct SelectorEdgeFinishInputs<'a> {
    test_policy: crate::playwright::playwright_tests::TestPolicy,
    selector_regexes: &'a crate::playwright::selectors::SelectorRegexes,
    app_selectors: &'a [crate::playwright::selectors::AppSelector],
    prepared: Vec<crate::playwright::analysis::pipeline_occurrences::PreparedTestFile>,
    demand: crate::playwright::analysis::pipeline_occurrences::TestOccurrenceDemand,
}

fn finish_selector_edges_for_settings(
    root: &Path,
    settings: &crate::playwright::config::Settings,
    inputs: &PlaywrightSelectorEdgeInputs<'_>,
    finish: SelectorEdgeFinishInputs<'_>,
) -> Result<Vec<Edge>> {
    let routes = crate::playwright::analysis::pipeline_setup::collect_playwright_routes(
        root,
        settings,
        false,
        finish.demand.routes,
        inputs.facts,
        inputs.snapshot,
    )?;
    let route_idx = crate::playwright::analysis::routes_index::route_index(root, routes.as_slice());
    let app_selector_targets = crate::playwright::analysis::selectors_index::app_selector_targets(
        root,
        finish.app_selectors,
    );
    let selector_idx =
        crate::playwright::analysis::selectors_index::selector_index(&app_selector_targets);
    let context = crate::playwright::analysis::context::TestAnalysisContext {
        root,
        route_index: &route_idx,
        selector_index: &selector_idx,
        navigation_helpers: &settings.navigation_helpers,
        selector_wrappers: &settings.selector_wrappers,
        selector_regexes: finish.selector_regexes,
        test_policy: finish.test_policy,
    };
    let pending = crate::playwright::analysis::pipeline_test_analysis::analyze_direct_test_files(
        finish.prepared,
        &context,
    );
    let mut test_analysis = finish_selector_text_edges(SelectorTextEdgeInputs {
        root,
        settings,
        inputs,
        routes: routes.as_slice(),
        context: &context,
        pending,
        has_eligible_text_locator: finish.demand.text_locators,
        test_policy: finish.test_policy,
    })?;
    test_analysis.edges.sort();
    test_analysis.edges.dedup();
    Ok(selector_edges_from_playwright_edges(
        root,
        inputs.all_files,
        &test_analysis.edges,
        inputs.interner,
    ))
}
