use crate::playwright::analysis::app_collect::collect_app_selector_occurrences;
use crate::playwright::analysis::app_text::collect_app_text_targets;
use crate::playwright::analysis::context::TestAnalysisContext;
use crate::playwright::analysis::coverage::build_coverage;
use crate::playwright::analysis::discover::discover_test_files;
use crate::playwright::analysis::fetch::{collect_fetches_for_routes, expand_fetch_edges};
pub(crate) use crate::playwright::analysis::pipeline_selectors::{
    analyze_selectors_with_policy, analyze_selectors_with_policy_and_facts,
};
use crate::playwright::analysis::pipeline_test_analysis::analyze_test_files;
use crate::playwright::analysis::route_reachability::collect_route_reachable_files;
use crate::playwright::analysis::routes_index::route_index;
use crate::playwright::analysis::selectors_index::{app_selector_targets, selector_index};
use crate::playwright::analysis::types::{
    Analysis, CoverageInputs, EdgeReport, UniqueSelectorPolicy,
};
use crate::playwright::config;
use crate::playwright::config::has_configured_html_id_selector;
use crate::playwright::playwright_tests;
use crate::playwright::routes;
use crate::playwright::selectors;
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
    )
}

pub(crate) fn analyze_with_policy_and_optional_facts(
    root: &Path,
    settings: &config::Settings,
    test_policy: playwright_tests::TestPolicy,
    mut unique_selector_policy: UniqueSelectorPolicy,
    require_routes: bool,
    facts: Option<&dyn crate::codebase::dependencies::graph::TsFactLookup>,
) -> Result<Analysis> {
    unique_selector_policy.configured_html_id_selector = has_configured_html_id_selector(settings);
    let route_root = root.join(&settings.frontend_root);
    let mut routes =
        crate::perf_trace::trace("playwright.routes", || routes::collect_routes(&route_root));
    let virtual_routes = crate::routes::rewrites::expand_rewrites(&settings.rewrites, &routes);
    routes.extend(virtual_routes);
    if require_routes && routes.is_empty() {
        let route_display = route_root.strip_prefix(root).unwrap_or(&route_root);
        anyhow::bail!(
            "no Next.js page routes found under {}",
            route_display.display()
        );
    }

    let playwright = crate::playwright::playwright_config::load_many(
        root,
        &settings.playwright_configs,
        settings.project.as_deref(),
    )?;
    let test_files = crate::perf_trace::trace("playwright.discover_test_files", || {
        discover_test_files(root, settings, &playwright)
    })?;
    let selector_regexes = selectors::compile_selector_regexes_with_html_ids(
        &settings.selector_attributes,
        &settings.component_selector_attributes,
        settings.html_ids,
    );
    let unique_html_id_scan = unique_selector_policy.html_ids && !settings.html_ids;
    let app_selector_regexes = selectors::compile_selector_regexes_with_html_ids(
        &settings.selector_attributes,
        &settings.component_selector_attributes,
        settings.html_ids || unique_html_id_scan,
    );
    let app_selector_occurrences = if settings.selector_attributes.is_empty()
        && settings.component_selector_attributes.is_empty()
        && !settings.html_ids
        && !unique_html_id_scan
    {
        Vec::new()
    } else {
        crate::perf_trace::trace("playwright.app_selector_occurrences", || {
            collect_app_selector_occurrences(root, settings, &app_selector_regexes)
        })?
    };
    let mut app_selectors: Vec<_> = app_selector_occurrences
        .iter()
        .filter(|selector| {
            settings.html_ids
                || unique_selector_policy.configured_html_id_selector
                || selector.attribute != selectors::HTML_ID_ATTRIBUTE
        })
        .cloned()
        .collect();
    app_selectors.sort();
    app_selectors.dedup();
    let app_text_targets = crate::perf_trace::trace("playwright.app_text_targets", || {
        collect_app_text_targets(root, settings)
    })?;
    let route_reachable_files = if app_text_targets.is_empty() {
        Default::default()
    } else {
        crate::perf_trace::trace("playwright.route_reachable_files", || {
            collect_route_reachable_files(root, settings, &routes)
        })?
    };
    let route_idx = route_index(root, &routes);
    let app_selector_tgts = app_selector_targets(root, &app_selectors);
    let selector_idx = selector_index(&app_selector_tgts);
    let test_analysis = TestAnalysisContext {
        root,
        route_index: &route_idx,
        app_selector_targets: &app_selector_tgts,
        selector_index: &selector_idx,
        app_text_targets: &app_text_targets,
        route_reachable_files: &route_reachable_files,
        navigation_helpers: &settings.navigation_helpers,
        selector_regexes: &selector_regexes,
        test_policy,
    };

    let mut test_analysis_result =
        crate::perf_trace::trace("playwright.test_file_analysis", || {
            analyze_test_files(&test_files, &test_analysis, facts)
        })?;

    let mut edges = test_analysis_result.edges;
    test_analysis_result.helper_references.sort();
    test_analysis_result.helper_references.dedup();

    let fetch_idx = if routes.is_empty() {
        Default::default()
    } else {
        crate::perf_trace::trace("playwright.fetches_for_routes", || {
            collect_fetches_for_routes(&routes, &route_root, root)
        })?
    };
    edges.extend(expand_fetch_edges(&edges, &fetch_idx));
    edges.sort();
    edges.dedup();

    let edge_report = EdgeReport { edges };
    let coverage = crate::perf_trace::trace("playwright.build_coverage", || {
        build_coverage(CoverageInputs {
            root,
            routes: &routes,
            app_selectors: &app_selectors,
            app_selector_occurrences: &app_selector_occurrences,
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
