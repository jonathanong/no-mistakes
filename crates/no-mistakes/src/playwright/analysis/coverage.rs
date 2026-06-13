use crate::playwright::analysis::coverage_edges::seed_coverage_from_edges;
use crate::playwright::analysis::duplicates::build_duplicate_selectors;
use crate::playwright::analysis::fetch::{seed_fetch_coverage, FetchCoverageEntry};
use crate::playwright::analysis::types::{
    CoverageFetch, CoverageInputs, CoverageLinks, CoverageReport, CoverageRoute, CoverageSelector,
    SelectorCoverageKey, Summary, TestRef,
};
use crate::playwright::fsutil::relative_string;
use crate::playwright::url::is_ignored;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

pub(super) type RouteCoverageEntry<'a> = BTreeMap<
    &'a str,
    (
        BTreeSet<Arc<String>>,
        BTreeSet<Arc<String>>,
        BTreeSet<TestRef>,
    ),
>;
pub(super) type SelectorCoverageEntry =
    BTreeMap<SelectorCoverageKey, (CoverageLinks, BTreeSet<TestRef>)>;

pub(crate) fn build_coverage(inputs: CoverageInputs<'_>) -> CoverageReport {
    let root = inputs.root;
    let routes = inputs.routes;
    let app_selectors = inputs.app_selectors;
    let app_selector_occurrences = inputs.app_selector_occurrences;
    let edges = inputs.edges;
    let helper_references = inputs.helper_references;
    let settings = inputs.settings;
    let unique_selector_policy = inputs.unique_selector_policy;
    let fetch_index = inputs.fetch_index;

    let ignored: Vec<String> = settings.ignore_routes.clone();
    let mut by_route: RouteCoverageEntry<'_> = BTreeMap::new();
    let mut by_selector: SelectorCoverageEntry = BTreeMap::new();
    let mut by_fetch: FetchCoverageEntry = seed_fetch_coverage(fetch_index);

    seed_coverage_from_edges(edges, &mut by_route, &mut by_selector, &mut by_fetch);

    let mut coverage_routes: Vec<CoverageRoute> = Vec::new();
    for route in routes {
        let (tests, urls, tests_detail) = by_route
            .get(route.pattern.as_str())
            .cloned()
            .unwrap_or_default();
        let covered = !tests.is_empty() || is_ignored(&route.pattern, &ignored);
        coverage_routes.push(CoverageRoute {
            route: route.pattern.clone(),
            file: relative_string(root, &route.file),
            covered,
            tests: tests.into_iter().map(|test| test.to_string()).collect(),
            tests_detail: tests_detail.into_iter().collect(),
            urls: urls.into_iter().map(|url| url.to_string()).collect(),
        });
    }

    coverage_routes.sort_by(|a, b| a.route.cmp(&b.route).then_with(|| a.file.cmp(&b.file)));
    let mut helper_references_by_selector = BTreeMap::<(String, String), Vec<_>>::new();
    for helper_reference in helper_references {
        helper_references_by_selector
            .entry((
                helper_reference.attribute.clone(),
                helper_reference.value.clone(),
            ))
            .or_default()
            .push(helper_reference.reference.clone());
    }

    let mut coverage_selectors: Vec<CoverageSelector> = Vec::new();
    for app_selector in app_selectors {
        let app_file = Arc::new(relative_string(root, &app_selector.file));
        let value = app_selector.display_value();
        let attribute = app_selector.attribute.clone();
        let key = (app_file.clone(), attribute.clone(), value.clone());
        let ((tests, selectors), tests_detail) = by_selector.get(&key).cloned().unwrap_or_default();
        let covered = !tests.is_empty();
        let unsupported_dynamic = app_selector.unsupported_dynamic();
        let helper_references = if covered || unsupported_dynamic {
            Vec::new()
        } else {
            helper_references_by_selector
                .get(&(attribute.clone(), value.clone()))
                .cloned()
                .unwrap_or_default()
        };
        coverage_selectors.push(CoverageSelector {
            attribute,
            value,
            file: app_file.to_string(),
            covered,
            unsupported_dynamic,
            tests: tests.into_iter().map(|test| test.to_string()).collect(),
            tests_detail: tests_detail.into_iter().collect(),
            selectors: selectors
                .into_iter()
                .map(|selector| selector.to_string())
                .collect(),
            helper_references,
        });
    }
    coverage_selectors.sort_by(|a, b| {
        a.attribute
            .cmp(&b.attribute)
            .then_with(|| a.value.cmp(&b.value))
            .then_with(|| a.file.cmp(&b.file))
    });

    let mut fetch_apis: Vec<CoverageFetch> = by_fetch
        .into_iter()
        .map(
            |((method, path), (tests, tests_detail, route_files))| CoverageFetch {
                covered: !tests.is_empty(),
                tests: tests.into_iter().map(|test| test.to_string()).collect(),
                tests_detail: tests_detail.into_iter().collect(),
                route_files: route_files
                    .into_iter()
                    .map(|route_file| route_file.to_string())
                    .collect(),
                method,
                path,
            },
        )
        .collect();
    fetch_apis.sort_by(|a, b| a.method.cmp(&b.method).then_with(|| a.path.cmp(&b.path)));

    let total_routes = coverage_routes.len();
    let covered_routes = coverage_routes.iter().filter(|route| route.covered).count();
    let uncovered_routes = total_routes.saturating_sub(covered_routes);
    let total_selectors = coverage_selectors.len();
    let covered_selectors = coverage_selectors
        .iter()
        .filter(|selector| selector.covered)
        .count();
    let uncovered_selectors = total_selectors.saturating_sub(covered_selectors);
    let duplicate_selectors =
        build_duplicate_selectors(root, app_selector_occurrences, unique_selector_policy);
    let duplicate_selector_count = duplicate_selectors.len();
    let total_fetch_apis = fetch_apis.len();
    let covered_fetch_apis = fetch_apis.iter().filter(|f| f.covered).count();
    let uncovered_fetch_apis = total_fetch_apis.saturating_sub(covered_fetch_apis);

    CoverageReport {
        summary: Summary {
            total_routes,
            covered_routes,
            uncovered_routes,
            total_selectors,
            covered_selectors,
            uncovered_selectors,
            duplicate_selectors: duplicate_selector_count,
            total_fetch_apis,
            covered_fetch_apis,
            uncovered_fetch_apis,
        },
        routes: coverage_routes,
        selectors: coverage_selectors,
        duplicate_selectors,
        fetch_apis,
    }
}
