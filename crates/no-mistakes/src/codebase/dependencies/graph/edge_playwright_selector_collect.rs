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
    let mut test_analysis = finish_selector_text_edges(
        root,
        settings,
        inputs,
        routes.as_slice(),
        &context,
        pending,
        finish.demand.text_locators,
        finish.test_policy,
    )?;
    test_analysis.edges.sort();
    test_analysis.edges.dedup();
    Ok(selector_edges_from_playwright_edges(
        root,
        inputs.all_files,
        &test_analysis.edges,
        inputs.interner,
    ))
}

fn finish_selector_text_edges(
    root: &Path,
    settings: &crate::playwright::config::Settings,
    inputs: &PlaywrightSelectorEdgeInputs<'_>,
    routes: &[crate::routes::Route],
    context: &crate::playwright::analysis::context::TestAnalysisContext<'_>,
    pending: Vec<crate::playwright::analysis::pipeline_test_analysis::PendingTestFileAnalysis>,
    has_eligible_text_locator: bool,
    test_policy: crate::playwright::playwright_tests::TestPolicy,
) -> Result<crate::playwright::analysis::types::TestFileAnalysis> {
    let text_setup = crate::playwright::analysis::pipeline_text_setup::build_text_resolution_setup(
        root,
        settings,
        crate::playwright::analysis::pipeline_text_setup::TextResolutionInputs {
            facts: inputs.facts,
            graph_file_universe: Some(inputs.all_files),
            route_import_candidate: inputs.route_import(),
            routes,
            snapshot: inputs.snapshot,
            has_eligible_text_locator,
            has_text_candidate: &|targets, index| {
                crate::playwright::analysis::pipeline_test_analysis::has_text_locator_candidate(
                    &pending,
                    targets,
                    index,
                    test_policy,
                )
            },
            has_route_reachability_demand: &|targets, index| {
                crate::playwright::analysis::pipeline_test_analysis::has_route_reachability_demand(
                    root,
                    &pending,
                    targets,
                    index,
                    test_policy,
                )
            },
        },
    )?;
    let text_context = text_setup.has_matching_text_candidate.then_some(
        crate::playwright::analysis::text_edges::TextEdgeContext {
            app_text_targets: text_setup.app_text_targets.as_slice(),
            app_text_index: &text_setup.app_text_index,
            route_reachable_files: &text_setup.route_reachable_files,
            test_policy,
        },
    );
    Ok(
        crate::playwright::analysis::pipeline_test_analysis::finish_test_file_analysis(
            pending,
            context,
            text_context.as_ref(),
        ),
    )
}

fn selector_wrapper_resolution(
    root: &Path,
    settings: &crate::playwright::config::Settings,
    snapshot: &crate::playwright::fsutil::VisiblePathSnapshot,
    route_import_candidate: Option<(&DepGraph, &crate::codebase::ts_resolver::TsConfig)>,
) -> Result<Option<crate::codebase::check_facts::PlaywrightModuleResolution>> {
    if settings.selector_wrappers.is_empty() {
        return Ok(None);
    }
    let paths = snapshot.paths_for(root);
    let sources = snapshot.source_store_for(root);
    let tsconfig = match route_import_candidate {
        Some((_, tsconfig)) => tsconfig.clone(),
        None => crate::codebase::ts_resolver::resolve_tsconfig_from_visible_and_sources(
            None, root, &paths, &sources,
        )?,
    };
    let workspace = crate::codebase::workspaces::load_indexed_from_source_store(root, &sources)
        .unwrap_or_default();
    Ok(Some(
        crate::codebase::check_facts::PlaywrightModuleResolution::new(
            Arc::new(tsconfig),
            Arc::new(workspace),
            Arc::new(paths.iter().cloned().collect()),
        ),
    ))
}
