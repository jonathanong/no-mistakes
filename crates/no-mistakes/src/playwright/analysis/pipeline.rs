use crate::playwright::analysis::context::TestAnalysisContext;
pub(crate) use crate::playwright::analysis::pipeline_entrypoints::{
    analyze_with_policy_and_facts_from_snapshot, analyze_with_policy_from_snapshot,
};
pub(crate) use crate::playwright::analysis::pipeline_facts::{
    standalone_fact_plan, standalone_facts,
};
use crate::playwright::analysis::pipeline_occurrences::prepare_test_files;
use crate::playwright::analysis::pipeline_options::AnalysisOptions;
pub(crate) use crate::playwright::analysis::pipeline_selectors::{
    analyze_selectors_with_policy_and_facts_from_snapshot,
    analyze_selectors_with_policy_from_snapshot,
};
use crate::playwright::analysis::pipeline_setup::{
    collect_app_selectors, collect_playwright_routes, discover_playwright_test_files,
    PlaywrightSetup,
};
use crate::playwright::analysis::pipeline_test_analysis::{
    analyze_direct_test_files, finish_test_file_analysis, has_route_reachability_demand,
    has_text_locator_candidate,
};
use crate::playwright::analysis::pipeline_text_setup::{
    build_text_resolution_setup, TextResolutionInputs,
};
use crate::playwright::analysis::routes_index::route_index;
use crate::playwright::analysis::selectors_index::{app_selector_targets, selector_index};
use crate::playwright::analysis::text_edges::TextEdgeContext;
use crate::playwright::analysis::types::{Analysis, UniqueSelectorPolicy};
use crate::playwright::{config, playwright_tests, selectors};
use anyhow::Result;
use std::path::Path;

#[cfg(test)]
pub(crate) mod test_support;

pub(crate) fn analyze_with_policy_and_optional_facts(
    root: &Path,
    settings: &config::Settings,
    test_policy: playwright_tests::TestPolicy,
    mut unique_selector_policy: UniqueSelectorPolicy,
    options: AnalysisOptions<'_>,
) -> Result<Analysis> {
    let AnalysisOptions {
        require_routes,
        skip_test_file_errors,
        facts,
        route_import_candidate,
        graph_file_universe,
        occurrence_selection,
        snapshot,
    } = options;
    unique_selector_policy.configured_html_id_selector =
        config::has_configured_html_id_selector(settings);
    let selector_regexes = selectors::compile_selector_regexes_with_html_ids(
        &settings.selector_attributes,
        &settings.component_selector_attributes,
        settings.html_ids,
    );
    let required_routes = require_routes
        .then(|| collect_playwright_routes(root, settings, true, false, facts, snapshot))
        .transpose()?;
    let test_files = discover_playwright_test_files(root, settings, facts, snapshot)?;
    let app_selector_setup =
        collect_app_selectors(root, settings, &unique_selector_policy, facts, snapshot)?;
    let (prepared, demand) = crate::perf_trace::trace("playwright.test_occurrences", || {
        prepare_test_files(
            test_files,
            settings,
            &selector_regexes,
            test_policy,
            skip_test_file_errors,
            facts,
            occurrence_selection,
        )
    })?;
    let routes = match required_routes {
        Some(routes) => routes,
        None => collect_playwright_routes(root, settings, false, demand.routes, facts, snapshot)?,
    };
    let setup = PlaywrightSetup {
        routes,
        app_selectors: app_selector_setup.app_selectors,
        app_selector_occurrences: app_selector_setup.app_selector_occurrences,
    };
    let route_idx = route_index(root, setup.routes.as_slice());
    let app_selector_targets = app_selector_targets(root, &setup.app_selectors);
    let selector_idx = selector_index(&app_selector_targets);
    let direct_context = TestAnalysisContext {
        root,
        route_index: &route_idx,
        selector_index: &selector_idx,
        navigation_helpers: &settings.navigation_helpers,
        selector_regexes: &selector_regexes,
        test_policy,
    };
    let pending = crate::perf_trace::trace("playwright.test_direct_edges", || {
        analyze_direct_test_files(prepared, &direct_context)
    });
    let text_setup = build_text_resolution_setup(
        root,
        settings,
        TextResolutionInputs {
            facts,
            graph_file_universe,
            route_import_candidate,
            routes: setup.routes.as_slice(),
            snapshot,
            has_eligible_text_locator: demand.text_locators,
            has_text_candidate: &|app_text_targets, app_text_index| {
                has_text_locator_candidate(&pending, app_text_targets, app_text_index, test_policy)
            },
            has_route_reachability_demand: &|app_text_targets, app_text_index| {
                has_route_reachability_demand(
                    root,
                    &pending,
                    app_text_targets,
                    app_text_index,
                    test_policy,
                )
            },
        },
    )?;
    let text_context = text_setup
        .has_matching_text_candidate
        .then_some(TextEdgeContext {
            app_text_targets: text_setup.app_text_targets.as_slice(),
            app_text_index: &text_setup.app_text_index,
            route_reachable_files: &text_setup.route_reachable_files,
            test_policy,
        });
    let mut test_analysis = crate::perf_trace::trace("playwright.test_text_edges", || {
        finish_test_file_analysis(pending, &direct_context, text_context.as_ref())
    });
    test_analysis.helper_references.sort();
    test_analysis.helper_references.dedup();
    super::pipeline_finish::finish_analysis(
        root,
        settings,
        unique_selector_policy,
        setup,
        test_analysis,
        facts,
        snapshot,
    )
}
