use crate::playwright::analysis::context::TestAnalysisContext;
use crate::playwright::analysis::coverage::build_coverage;
use crate::playwright::analysis::fetch::{collect_fetches_for_routes, expand_fetch_edges};
pub(crate) use crate::playwright::analysis::pipeline_selectors::{
    analyze_selectors_with_policy, analyze_selectors_with_policy_and_facts,
};
use crate::playwright::analysis::pipeline_setup::build_playwright_setup;
use crate::playwright::analysis::pipeline_test_analysis::analyze_test_files;
use crate::playwright::analysis::routes_index::route_index;
use crate::playwright::analysis::selectors_index::{app_selector_targets, selector_index};
use crate::playwright::analysis::types::{
    Analysis, CoverageInputs, EdgeReport, UniqueSelectorPolicy,
};
use crate::playwright::config;
use crate::playwright::config::has_configured_html_id_selector;
use crate::playwright::playwright_tests;
use anyhow::Result;
use std::path::Path;
pub(crate) fn analyze_with_policy(
    root: &Path,
    settings: &config::Settings,
    test_policy: playwright_tests::TestPolicy,
    unique_selector_policy: UniqueSelectorPolicy,
) -> Result<Analysis> {
    analyze_with_policy_and_optional_facts(
        root,
        settings,
        test_policy,
        unique_selector_policy,
        true,
        None,
        None,
    )
}

pub(crate) fn analyze_with_policy_and_facts(
    root: &Path,
    settings: &config::Settings,
    test_policy: playwright_tests::TestPolicy,
    unique_selector_policy: UniqueSelectorPolicy,
    facts: &dyn crate::codebase::dependencies::graph::TsFactLookup,
) -> Result<Analysis> {
    analyze_with_policy_and_optional_facts(
        root,
        settings,
        test_policy,
        unique_selector_policy,
        true,
        Some(facts),
        None,
    )
}

pub(crate) fn analyze_with_policy_and_optional_facts(
    root: &Path,
    settings: &config::Settings,
    test_policy: playwright_tests::TestPolicy,
    mut unique_selector_policy: UniqueSelectorPolicy,
    require_routes: bool,
    facts: Option<&dyn crate::codebase::dependencies::graph::TsFactLookup>,
    route_import_graph: Option<&crate::codebase::dependencies::graph::DepGraph>,
) -> Result<Analysis> {
    unique_selector_policy.configured_html_id_selector = has_configured_html_id_selector(settings);
    let route_root = root.join(&settings.frontend_root);
    let setup = build_playwright_setup(
        root,
        settings,
        &unique_selector_policy,
        require_routes,
        facts,
        route_import_graph,
    )?;

    let route_idx = route_index(root, setup.routes.as_slice());
    let app_selector_tgts = app_selector_targets(root, &setup.app_selectors);
    let selector_idx = selector_index(&app_selector_tgts);
    let test_analysis = TestAnalysisContext {
        root,
        route_index: &route_idx,
        app_selector_targets: &app_selector_tgts,
        selector_index: &selector_idx,
        app_text_targets: setup.app_text_targets.as_slice(),
        route_reachable_files: &setup.route_reachable_files,
        navigation_helpers: &settings.navigation_helpers,
        selector_regexes: &setup.selector_regexes,
        test_policy,
    };

    let mut test_analysis_result =
        crate::perf_trace::trace("playwright.test_file_analysis", || {
            analyze_test_files(&setup.test_files, &test_analysis, facts)
        })?;

    let mut edges = test_analysis_result.edges;
    test_analysis_result.helper_references.sort();
    test_analysis_result.helper_references.dedup();

    let fetch_idx = if setup.routes.is_empty() {
        Default::default()
    } else {
        crate::perf_trace::trace("playwright.fetches_for_routes", || {
            collect_fetches_for_routes(setup.routes.as_slice(), &route_root, root)
        })?
    };
    edges.extend(expand_fetch_edges(&edges, &fetch_idx));
    edges.sort();
    edges.dedup();

    let edge_report = EdgeReport { edges };
    let coverage = crate::perf_trace::trace("playwright.build_coverage", || {
        build_coverage(CoverageInputs {
            root,
            routes: setup.routes.as_slice(),
            app_selectors: &setup.app_selectors,
            app_selector_occurrences: setup.app_selector_occurrences.as_slice(),
            edges: &edge_report.edges,
            helper_references: &test_analysis_result.helper_references,
            settings,
            unique_selector_policy,
            fetch_index: &fetch_idx,
        })
    });
    Ok(Analysis {
        coverage,
        edges: edge_report,
    })
}
