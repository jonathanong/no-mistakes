use crate::playwright::analysis::app_collect::collect_app_selector_occurrences;
use crate::playwright::analysis::context::TestAnalysisContext;
use crate::playwright::analysis::coverage::build_coverage;
use crate::playwright::analysis::discover::discover_test_files;
use crate::playwright::analysis::fetch::{collect_fetches_for_routes, expand_fetch_edges};
use crate::playwright::analysis::routes_index::route_index;
use crate::playwright::analysis::selectors_index::{app_selector_targets, selector_index};
use crate::playwright::analysis::test_file::{analyze_test_file, analyze_test_occurrences};
use crate::playwright::analysis::types::{
    Analysis, CoverageInputs, EdgeReport, UniqueSelectorPolicy,
};
use crate::playwright::config;
use crate::playwright::config::has_configured_html_id_selector;
use crate::playwright::playwright_tests;
use crate::playwright::routes;
use crate::playwright::selectors;
use anyhow::Result;
use rayon::prelude::*;
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
        None,
    )
}

pub(crate) fn analyze_with_policy_and_facts(
    root: &Path,
    settings: &config::Settings,
    test_policy: playwright_tests::TestPolicy,
    unique_selector_policy: UniqueSelectorPolicy,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Analysis> {
    analyze_with_policy_and_optional_facts(
        root,
        settings,
        test_policy,
        unique_selector_policy,
        Some(facts),
    )
}

fn analyze_with_policy_and_optional_facts(
    root: &Path,
    settings: &config::Settings,
    test_policy: playwright_tests::TestPolicy,
    mut unique_selector_policy: UniqueSelectorPolicy,
    facts: Option<&crate::codebase::check_facts::CheckFactMap>,
) -> Result<Analysis> {
    unique_selector_policy.configured_html_id_selector = has_configured_html_id_selector(settings);
    let route_root = root.join(&settings.frontend_root);
    let routes = routes::collect_routes(&route_root);
    if routes.is_empty() {
        anyhow::bail!(
            "no Next.js page routes found under {}",
            route_root
                .strip_prefix(root)
                .unwrap_or(&route_root)
                .display()
        );
    }

    let playwright = crate::playwright::playwright_config::load_many(
        root,
        &settings.playwright_configs,
        settings.project.as_deref(),
    )?;
    let test_files = discover_test_files(root, settings, &playwright)?;
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
        collect_app_selector_occurrences(root, settings, &app_selector_regexes)?
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
    let route_idx = route_index(root, &routes);
    let app_selector_tgts = app_selector_targets(root, &app_selectors);
    let selector_idx = selector_index(&app_selector_tgts);
    let test_analysis = TestAnalysisContext {
        root,
        route_index: &route_idx,
        app_selector_targets: &app_selector_tgts,
        selector_index: &selector_idx,
        navigation_helpers: &settings.navigation_helpers,
        selector_regexes: &selector_regexes,
        test_policy,
    };

    let mut edges = test_files
        .par_iter()
        .try_fold(Vec::new, |mut edges, test_file| -> Result<_> {
            let test_edges = if let Some(facts) = facts {
                match facts
                    .ts
                    .get(&test_file.path)
                    .and_then(|file_facts| file_facts.playwright.as_ref())
                {
                    Some(playwright) => analyze_test_occurrences(
                        test_file,
                        &test_analysis,
                        playwright.urls.clone(),
                        playwright.selectors.clone(),
                    ),
                    None => analyze_test_file(test_file, &test_analysis)?,
                }
            } else {
                analyze_test_file(test_file, &test_analysis)?
            };
            edges.extend(test_edges);
            Ok(edges)
        })
        .try_reduce(Vec::new, |mut left, mut right| -> Result<_> {
            left.append(&mut right);
            Ok(left)
        })?;

    let fetch_idx = collect_fetches_for_routes(&routes, &route_root, root)?;
    edges.extend(expand_fetch_edges(&edges, &fetch_idx));
    edges.sort();
    edges.dedup();

    let edge_report = EdgeReport { edges };
    let coverage = build_coverage(CoverageInputs {
        root,
        routes: &routes,
        app_selectors: &app_selectors,
        app_selector_occurrences: &app_selector_occurrences,
        edges: &edge_report.edges,
        settings,
        unique_selector_policy,
        fetch_index: &fetch_idx,
    });
    Ok(Analysis {
        coverage,
        edges: edge_report,
    })
}
